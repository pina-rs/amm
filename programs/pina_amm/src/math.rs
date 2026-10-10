//! Pure, checked constant-product arithmetic.
//!
//! Every function in this module is deterministic and allocation-free, and
//! every rounding decision favours the pool:
//!
//! - fees round **up**;
//! - the amount a trader receives rounds **down**;
//! - the amount a trader pays rounds **up**;
//! - the protocol's and creator's shares of a fee round **down**, so the LPs
//!   keep the remainder.
//!
//! The same functions run on chain and in the generated SDKs' test vectors, so
//! a client can quote a trade exactly before it signs.
//!
//! ## Swap formulas
//!
//! For input reserve `x`, output reserve `y`, trade fee rate `t`, creator fee
//! rate `c`, and denominator `D = 1_000_000`:
//!
//! Exact input `a`, creator fee on the input token:
//!
//! ```text
//! fee      = ceil(a * (t + c) / D)
//! creator  = floor(fee * c / (t + c))
//! trade    = fee - creator
//! out      = floor((a - fee) * y / (x + a - fee))
//! ```
//!
//! Exact input `a`, creator fee on the output token:
//!
//! ```text
//! trade    = ceil(a * t / D)
//! gross    = floor((a - trade) * y / (x + a - trade))
//! creator  = ceil(gross * c / D)
//! out      = gross - creator
//! ```
//!
//! Exact output works backwards through the same steps with every division
//! rounded up. In all cases the protocol takes `floor(trade * p / D)` of the
//! trade fee and the rest of the trade fee stays in the pool for the LPs.

use pina::ProgramError;

use crate::errors::AmmError;
use crate::state::FEE_RATE_DENOMINATOR;
use crate::state::MAX_SWAP_FEE_RATE;
use crate::state::MINIMUM_LIQUIDITY;

const DENOMINATOR: u128 = FEE_RATE_DENOMINATOR as u128;

/// The three fee rates a pool snapshots from its tier, in parts per million.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FeeRates {
	/// Fee charged on every swap; the LPs keep what the protocol does not.
	pub trade: u32,
	/// Protocol share of the trade fee.
	pub protocol: u32,
	/// Additional fee paid to the pool creator.
	pub creator: u32,
}

impl FeeRates {
	/// Reject rates a tier may not store.
	///
	/// The trade and creator fee together may not exceed
	/// [`MAX_SWAP_FEE_RATE`], and the protocol share may not exceed the whole
	/// trade fee.
	pub fn validate(self) -> Result<Self, ProgramError> {
		let swap_fee = self
			.trade
			.checked_add(self.creator)
			.ok_or(AmmError::InvalidFeeRates)?;
		if swap_fee > MAX_SWAP_FEE_RATE || self.protocol > FEE_RATE_DENOMINATOR {
			return Err(AmmError::InvalidFeeRates.into());
		}
		Ok(self)
	}
}

/// The reserves on the input and output side of one swap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapReserves {
	/// Reserve of the token the trader sells.
	pub input: u64,
	/// Reserve of the token the trader buys.
	pub output: u64,
}

/// Every amount one swap moves, computed before any token transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SwapQuote {
	/// Tokens debited from the trader.
	pub amount_in: u64,
	/// Tokens credited to the trader.
	pub amount_out: u64,
	/// Trade fee in the input token, including the protocol's share.
	pub trade_fee: u64,
	/// Protocol share of the trade fee, accrued in the input token.
	pub protocol_fee: u64,
	/// Creator fee, accrued in the input token when `creator_fee_on_input` is
	/// set and in the output token otherwise.
	pub creator_fee: u64,
	/// Whether `creator_fee` is denominated in the input token.
	pub creator_fee_on_input: bool,
}

impl SwapQuote {
	/// Reserves after this swap settles.
	///
	/// The input reserve grows by everything the trader paid except the fees
	/// accrued for the protocol and creator; the output reserve shrinks by
	/// what the trader received plus any creator fee taken from the output.
	pub fn reserves_after(self, before: SwapReserves) -> Result<SwapReserves, ProgramError> {
		let creator_input = if self.creator_fee_on_input {
			self.creator_fee
		} else {
			0
		};
		let creator_output = if self.creator_fee_on_input {
			0
		} else {
			self.creator_fee
		};
		let input = before
			.input
			.checked_add(self.amount_in)
			.and_then(|value| value.checked_sub(self.protocol_fee))
			.and_then(|value| value.checked_sub(creator_input))
			.ok_or(AmmError::MathOverflow)?;
		let output = before
			.output
			.checked_sub(self.amount_out)
			.and_then(|value| value.checked_sub(creator_output))
			.ok_or(AmmError::InsufficientLiquidity)?;
		Ok(SwapReserves { input, output })
	}
}

/// Convert a checked `u128` result into `u64`.
pub fn narrow(value: u128) -> Result<u64, ProgramError> {
	u64::try_from(value).map_err(|_| AmmError::MathOverflow.into())
}

/// `ceil(numerator / denominator)` for a non-zero denominator.
pub fn div_ceil(numerator: u128, denominator: u128) -> Result<u128, ProgramError> {
	if denominator == 0 {
		return Err(AmmError::MathOverflow.into());
	}
	Ok(numerator.div_ceil(denominator))
}

/// `ceil(amount * rate / 1_000_000)`: a fee rounded in the pool's favour.
pub fn fee_ceil(amount: u64, rate: u32) -> Result<u64, ProgramError> {
	narrow(div_ceil(
		u128::from(amount) * u128::from(rate),
		DENOMINATOR,
	)?)
}

/// `floor(amount * rate / 1_000_000)`: a share that leaves the remainder in
/// the pool.
pub fn share_floor(amount: u64, rate: u32) -> Result<u64, ProgramError> {
	narrow(u128::from(amount) * u128::from(rate) / DENOMINATOR)
}

/// `ceil(net * 1_000_000 / (1_000_000 - rate))`: the gross amount whose fee
/// at `rate` leaves at least `net`.
pub fn gross_up(net: u64, rate: u32) -> Result<u64, ProgramError> {
	let remaining = DENOMINATOR
		.checked_sub(u128::from(rate))
		.filter(|value| *value > 0)
		.ok_or(AmmError::InvalidFeeRates)?;
	narrow(div_ceil(u128::from(net) * DENOMINATOR, remaining)?)
}

/// Floor of the integer square root.
///
/// Uses the bit-by-bit method, which needs only shifts, comparisons, and
/// subtraction. A division-based search would pay for a software `u128`
/// division on every iteration on SBF.
#[must_use]
pub const fn sqrt_floor(mut value: u128) -> u128 {
	let mut result = 0u128;
	let mut bit = 1u128 << 126;

	while bit > value {
		bit >>= 2;
	}

	while bit != 0 {
		if value >= result + bit {
			value -= result + bit;
			result = (result >> 1) + bit;
		} else {
			result >>= 1;
		}
		bit >>= 2;
	}

	result
}

/// Output for `net_in` sold into the pool, rounded down.
///
/// `floor(net_in * y / (x + net_in))`.
pub fn constant_product_out(net_in: u64, reserves: SwapReserves) -> Result<u64, ProgramError> {
	let denominator = u128::from(reserves.input) + u128::from(net_in);
	if denominator == 0 {
		return Err(AmmError::InsufficientLiquidity.into());
	}
	narrow(u128::from(net_in) * u128::from(reserves.output) / denominator)
}

/// Input needed to take `gross_out` from the pool, rounded up.
///
/// `ceil(x * gross_out / (y - gross_out))`.
pub fn constant_product_in(gross_out: u64, reserves: SwapReserves) -> Result<u64, ProgramError> {
	if gross_out >= reserves.output {
		return Err(AmmError::InsufficientLiquidity.into());
	}
	let remaining = u128::from(reserves.output - gross_out);
	narrow(div_ceil(
		u128::from(reserves.input) * u128::from(gross_out),
		remaining,
	)?)
}

/// The creator's part of a combined fee, rounded down.
fn creator_share(total_fee: u64, rates: FeeRates) -> Result<u64, ProgramError> {
	let combined = u128::from(rates.trade) + u128::from(rates.creator);
	if combined == 0 {
		return Ok(0);
	}
	narrow(u128::from(total_fee) * u128::from(rates.creator) / combined)
}

/// Quote a swap that sells exactly `amount_in`.
///
/// # Errors
///
/// - [`AmmError::ZeroAmount`] when `amount_in` is zero or the trader would
///   receive nothing;
/// - [`AmmError::MathOverflow`] when an intermediate value overflows.
pub fn swap_exact_in(
	amount_in: u64,
	reserves: SwapReserves,
	rates: FeeRates,
	creator_fee_on_input: bool,
) -> Result<SwapQuote, ProgramError> {
	if amount_in == 0 {
		return Err(AmmError::ZeroAmount.into());
	}

	let (trade_fee, input_creator_fee) = if creator_fee_on_input {
		let total = fee_ceil(amount_in, rates.trade + rates.creator)?;
		let creator = creator_share(total, rates)?;
		(total - creator, creator)
	} else {
		(fee_ceil(amount_in, rates.trade)?, 0)
	};
	let net_in = amount_in
		.checked_sub(trade_fee + input_creator_fee)
		.ok_or(AmmError::ZeroAmount)?;
	let gross_out = constant_product_out(net_in, reserves)?;
	let output_creator_fee = if creator_fee_on_input {
		0
	} else {
		fee_ceil(gross_out, rates.creator)?
	};
	let amount_out = gross_out
		.checked_sub(output_creator_fee)
		.ok_or(AmmError::ZeroAmount)?;
	if amount_out == 0 {
		return Err(AmmError::ZeroAmount.into());
	}

	Ok(SwapQuote {
		amount_in,
		amount_out,
		trade_fee,
		protocol_fee: share_floor(trade_fee, rates.protocol)?,
		creator_fee: input_creator_fee + output_creator_fee,
		creator_fee_on_input,
	})
}

/// Quote a swap that buys exactly `amount_out`.
///
/// # Errors
///
/// - [`AmmError::ZeroAmount`] when `amount_out` is zero;
/// - [`AmmError::InsufficientLiquidity`] when the pool cannot pay
///   `amount_out` plus any creator fee taken from the output;
/// - [`AmmError::MathOverflow`] when the required input does not fit `u64`.
pub fn swap_exact_out(
	amount_out: u64,
	reserves: SwapReserves,
	rates: FeeRates,
	creator_fee_on_input: bool,
) -> Result<SwapQuote, ProgramError> {
	if amount_out == 0 {
		return Err(AmmError::ZeroAmount.into());
	}

	let quote = if creator_fee_on_input {
		let net_in = constant_product_in(amount_out, reserves)?;
		let amount_in = gross_up(net_in, rates.trade + rates.creator)?;
		let total = amount_in - net_in;
		let creator_fee = creator_share(total, rates)?;
		SwapQuote {
			amount_in,
			amount_out,
			trade_fee: total - creator_fee,
			protocol_fee: 0,
			creator_fee,
			creator_fee_on_input,
		}
	} else {
		let gross_out = gross_up(amount_out, rates.creator)?;
		let net_in = constant_product_in(gross_out, reserves)?;
		let amount_in = gross_up(net_in, rates.trade)?;
		SwapQuote {
			amount_in,
			amount_out,
			trade_fee: amount_in - net_in,
			protocol_fee: 0,
			creator_fee: gross_out - amount_out,
			creator_fee_on_input,
		}
	};

	Ok(SwapQuote {
		protocol_fee: share_floor(quote.trade_fee, rates.protocol)?,
		..quote
	})
}

/// Require that a swap did not decrease `x * y`.
///
/// Rounding already guarantees this; the check turns any future arithmetic
/// mistake into a failed transaction instead of a drained pool.
pub fn assert_constant_product(
	before: SwapReserves,
	after: SwapReserves,
) -> Result<(), ProgramError> {
	let k_before = u128::from(before.input) * u128::from(before.output);
	let k_after = u128::from(after.input) * u128::from(after.output);
	if k_after < k_before {
		return Err(AmmError::InvariantViolation.into());
	}
	Ok(())
}

/// One in Q64.64 fixed point, the unit of the price accumulator.
pub const Q64: u128 = 1 << 64;

/// Advance the time-weighted price accumulator of token 1 in token 0.
///
/// The accumulator sums `price_0 * elapsed_seconds` as Q64.64 price times
/// seconds, exactly like Uniswap V2: the price in force for an interval is
/// the pool's price at the start of it, so a caller samples the accumulator
/// at two times and divides the difference by the time difference. A pool
/// with an empty side records no price, and a zero `last_update` only
/// anchors the clock, so a first sample after a migration adds nothing.
pub fn advance_price_accumulator(
	reserve_0: u64,
	reserve_1: u64,
	cumulative: u128,
	last_update: u64,
	now: u64,
) -> Result<(u128, u64), ProgramError> {
	if now <= last_update {
		return Ok((cumulative, last_update));
	}
	let advanced = if last_update == 0 || reserve_0 == 0 || reserve_1 == 0 {
		cumulative
	} else {
		let price_0 = u128::from(reserve_1)
			.checked_mul(Q64)
			.ok_or(AmmError::MathOverflow)?
			/ u128::from(reserve_0);
		let elapsed = now - last_update;
		cumulative
			.checked_add(
				price_0
					.checked_mul(u128::from(elapsed))
					.ok_or(AmmError::MathOverflow)?,
			)
			.ok_or(AmmError::MathOverflow)?
	};
	Ok((advanced, now))
}

/// LP issued by a new pool and the part minted to its first depositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialLiquidity {
	/// Full economic LP supply: `floor(sqrt(amount_0 * amount_1))`.
	pub lp_supply: u64,
	/// LP minted to the depositor: `lp_supply - MINIMUM_LIQUIDITY`.
	pub lp_minted: u64,
}

/// LP for a pool's first deposit.
///
/// # Errors
///
/// [`AmmError::InsufficientInitialLiquidity`] when the geometric mean of the
/// deposit is not larger than [`MINIMUM_LIQUIDITY`].
pub fn initial_liquidity(amount_0: u64, amount_1: u64) -> Result<InitialLiquidity, ProgramError> {
	let lp_supply = narrow(sqrt_floor(u128::from(amount_0) * u128::from(amount_1)))?;
	if lp_supply <= MINIMUM_LIQUIDITY {
		return Err(AmmError::InsufficientInitialLiquidity.into());
	}
	Ok(InitialLiquidity {
		lp_supply,
		lp_minted: lp_supply - MINIMUM_LIQUIDITY,
	})
}

/// Token amounts for one side of a liquidity change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenAmounts {
	/// Amount of token 0.
	pub amount_0: u64,
	/// Amount of token 1.
	pub amount_1: u64,
}

/// Tokens a depositor must add to mint `lp_amount`, rounded up.
///
/// # Errors
///
/// [`AmmError::ZeroAmount`] when `lp_amount` or `lp_supply` is zero.
pub fn deposit_amounts(
	lp_amount: u64,
	lp_supply: u64,
	reserve_0: u64,
	reserve_1: u64,
) -> Result<TokenAmounts, ProgramError> {
	if lp_amount == 0 || lp_supply == 0 {
		return Err(AmmError::ZeroAmount.into());
	}
	let share = |reserve: u64| {
		narrow(div_ceil(
			u128::from(reserve) * u128::from(lp_amount),
			u128::from(lp_supply),
		)?)
	};
	Ok(TokenAmounts {
		amount_0: share(reserve_0)?,
		amount_1: share(reserve_1)?,
	})
}

/// Tokens returned for burning `lp_amount`, rounded down.
///
/// # Errors
///
/// - [`AmmError::ZeroAmount`] when `lp_amount` is zero or both outputs round
///   to zero;
/// - [`AmmError::InsufficientLiquidity`] when `lp_amount` would reach into the
///   permanently locked [`MINIMUM_LIQUIDITY`].
pub fn withdraw_amounts(
	lp_amount: u64,
	lp_supply: u64,
	reserve_0: u64,
	reserve_1: u64,
) -> Result<TokenAmounts, ProgramError> {
	if lp_amount == 0 {
		return Err(AmmError::ZeroAmount.into());
	}
	if lp_amount > lp_supply.saturating_sub(MINIMUM_LIQUIDITY) {
		return Err(AmmError::InsufficientLiquidity.into());
	}
	let share =
		|reserve: u64| narrow(u128::from(reserve) * u128::from(lp_amount) / u128::from(lp_supply));
	let amounts = TokenAmounts {
		amount_0: share(reserve_0)?,
		amount_1: share(reserve_1)?,
	};
	if amounts.amount_0 == 0 && amounts.amount_1 == 0 {
		return Err(AmmError::ZeroAmount.into());
	}
	Ok(amounts)
}

#[cfg(test)]
mod tests {
	extern crate std;

	use proptest::prelude::*;

	use super::*;

	const RATES: FeeRates = FeeRates {
		trade: 2_500,
		protocol: 200_000,
		creator: 500,
	};

	fn reserves(input: u64, output: u64) -> SwapReserves {
		SwapReserves { input, output }
	}

	fn code(error: ProgramError) -> AmmError {
		match error {
			ProgramError::Custom(code) => {
				match code {
					0 => AmmError::InvalidFeeRates,
					7 => AmmError::InsufficientInitialLiquidity,
					8 => AmmError::ZeroAmount,
					10 => AmmError::InsufficientLiquidity,
					11 => AmmError::MathOverflow,
					12 => AmmError::InvariantViolation,
					other => panic!("unexpected custom error {other}"),
				}
			}
			other => panic!("unexpected error {other:?}"),
		}
	}

	#[test]
	fn validates_fee_rates() {
		assert!(RATES.validate().is_ok());
		assert!(
			FeeRates {
				trade: MAX_SWAP_FEE_RATE,
				protocol: FEE_RATE_DENOMINATOR,
				creator: 0,
			}
			.validate()
			.is_ok()
		);
		for rates in [
			FeeRates {
				trade: MAX_SWAP_FEE_RATE,
				protocol: 0,
				creator: 1,
			},
			FeeRates {
				trade: 0,
				protocol: FEE_RATE_DENOMINATOR + 1,
				creator: 0,
			},
			FeeRates {
				trade: u32::MAX,
				protocol: 0,
				creator: 1,
			},
		] {
			assert_eq!(
				code(rates.validate().expect_err("rates must be rejected")),
				AmmError::InvalidFeeRates
			);
		}
	}

	#[test]
	fn rounds_fees_up_and_shares_down() {
		assert_eq!(fee_ceil(1_000_000, 2_500).expect("fee"), 2_500);
		assert_eq!(fee_ceil(1_000_001, 2_500).expect("fee"), 2_501);
		assert_eq!(fee_ceil(1, 1).expect("fee"), 1);
		assert_eq!(fee_ceil(0, 2_500).expect("fee"), 0);
		assert_eq!(share_floor(2_501, 200_000).expect("share"), 500);
		assert_eq!(gross_up(997_500, 2_500).expect("gross"), 1_000_000);
		assert_eq!(gross_up(997_501, 2_500).expect("gross"), 1_000_002);
	}

	#[test]
	fn square_root_is_exact_floor() {
		for value in [0u128, 1, 2, 3, 4, 15, 16, 17, 1 << 64, u128::MAX] {
			let root = sqrt_floor(value);
			assert!(root * root <= value);
			let next = root + 1;
			assert!(next.checked_mul(next).is_none_or(|square| square > value));
		}
	}

	#[test]
	fn exact_in_with_creator_fee_on_input_matches_hand_computation() {
		let quote =
			swap_exact_in(1_000_000, reserves(10_000_000, 20_000_000), RATES, true).expect("quote");
		// fee = ceil(1_000_000 * 3_000 / 1e6) = 3_000, creator = 3_000 * 500 / 3_000.
		assert_eq!(quote.trade_fee, 2_500);
		assert_eq!(quote.creator_fee, 500);
		assert_eq!(quote.protocol_fee, 500);
		// out = floor(997_000 * 20_000_000 / 10_997_000) = 1_813_221.
		assert_eq!(quote.amount_out, 1_813_221);
		assert!(quote.creator_fee_on_input);
	}

	#[test]
	fn exact_in_with_creator_fee_on_output_matches_hand_computation() {
		let quote = swap_exact_in(1_000_000, reserves(10_000_000, 20_000_000), RATES, false)
			.expect("quote");
		assert_eq!(quote.trade_fee, 2_500);
		// gross = floor(997_500 * 20_000_000 / 10_997_500) = 1_814_048.
		// creator = ceil(1_814_048 * 500 / 1e6) = 908.
		assert_eq!(quote.creator_fee, 908);
		assert_eq!(quote.amount_out, 1_814_048 - 908);
		assert!(!quote.creator_fee_on_input);
	}

	#[test]
	fn exact_in_rejects_dust_that_pays_nothing() {
		assert_eq!(
			code(swap_exact_in(0, reserves(1_000, 1_000), RATES, true).expect_err("zero")),
			AmmError::ZeroAmount
		);
		assert_eq!(
			code(swap_exact_in(1, reserves(1_000, 1_000), RATES, true).expect_err("dust")),
			AmmError::ZeroAmount
		);
	}

	#[test]
	fn exact_out_rejects_draining_the_pool() {
		assert_eq!(
			code(
				swap_exact_out(1_000, reserves(1_000, 1_000), RATES, true)
					.expect_err("whole reserve")
			),
			AmmError::InsufficientLiquidity
		);
		assert_eq!(
			code(swap_exact_out(0, reserves(1_000, 1_000), RATES, true).expect_err("zero")),
			AmmError::ZeroAmount
		);
	}

	#[test]
	fn liquidity_locks_the_minimum() {
		let liquidity = initial_liquidity(1_000_000, 4_000_000).expect("liquidity");
		assert_eq!(liquidity.lp_supply, 2_000_000);
		assert_eq!(liquidity.lp_minted, 2_000_000 - MINIMUM_LIQUIDITY);
		assert_eq!(
			code(initial_liquidity(1_000, 1_000).expect_err("too small")),
			AmmError::InsufficientInitialLiquidity
		);
	}

	#[test]
	fn withdrawals_cannot_reach_locked_liquidity() {
		assert_eq!(
			code(withdraw_amounts(1_001, 2_000, 2_000, 2_000).expect_err("locked liquidity")),
			AmmError::InsufficientLiquidity
		);
		let amounts = withdraw_amounts(1_000, 2_000, 2_000, 3_001).expect("withdraw");
		assert_eq!(
			amounts,
			TokenAmounts {
				amount_0: 1_000,
				amount_1: 1_500,
			}
		);
	}

	#[test]
	fn the_accumulator_sums_price_times_time() {
		let price = |r0: u64, r1: u64| u128::from(r1) * Q64 / u128::from(r0);
		// Ten seconds at 2:1, then ten at 4:1.
		let (cumulative, _) =
			advance_price_accumulator(100, 200, 0, 1_000, 1_010).expect("advance");
		assert_eq!(cumulative, price(100, 200) * 10);
		let (cumulative, _) =
			advance_price_accumulator(100, 400, cumulative, 1_010, 1_020).expect("advance");
		assert_eq!(cumulative, price(100, 200) * 10 + price(100, 400) * 10);
		// A zero start or an empty side anchors the clock without a price.
		assert_eq!(
			advance_price_accumulator(100, 200, 0, 0, 500).expect("anchor"),
			(0, 500)
		);
		assert_eq!(
			advance_price_accumulator(0, 200, 7, 500, 600).expect("empty"),
			(7, 600)
		);
		// Time moving backwards or standing still changes nothing.
		assert_eq!(
			advance_price_accumulator(100, 200, 7, 600, 599).expect("backwards"),
			(7, 600)
		);
	}

	#[test]
	fn deposits_round_up() {
		let amounts = deposit_amounts(1, 3, 10, 11).expect("deposit");
		assert_eq!(
			amounts,
			TokenAmounts {
				amount_0: 4,
				amount_1: 4,
			}
		);
	}

	fn rates() -> impl Strategy<Value = FeeRates> {
		(0u32..=60_000, 0u32..=FEE_RATE_DENOMINATOR, 0u32..=40_000).prop_map(
			|(trade, protocol, creator)| {
				FeeRates {
					trade,
					protocol,
					creator,
				}
			},
		)
	}

	proptest! {
		#![proptest_config(ProptestConfig::with_cases(2_048))]

		/// An exact-input swap never decreases `k`, never pays out the whole
		/// reserve, and charges fees that add up.
		#[test]
		fn exact_in_preserves_the_invariant(
			input in 1_000u64..=u64::MAX / 4,
			output in 1_000u64..=u64::MAX / 4,
			amount_in in 1u64..=u64::MAX / 4,
			rates in rates(),
			on_input in any::<bool>(),
		) {
			let before = reserves(input, output);
			if let Ok(quote) = swap_exact_in(amount_in, before, rates, on_input) {
				prop_assert!(quote.amount_out < output);
				prop_assert!(quote.protocol_fee <= quote.trade_fee);
				// A trade that would push a reserve past `u64::MAX` cannot settle
				// into a real vault, so it must be rejected rather than wrapped.
				match quote.reserves_after(before) {
					Ok(after) => prop_assert!(assert_constant_product(before, after).is_ok()),
					Err(error) => prop_assert_eq!(code(error), AmmError::MathOverflow),
				}
			}
		}

		/// An exact-output swap never decreases `k` and always delivers exactly
		/// the requested output.
		#[test]
		fn exact_out_preserves_the_invariant(
			input in 1_000u64..=u64::MAX / 4,
			output in 1_000u64..=u64::MAX / 4,
			amount_out in 1u64..=u64::MAX / 4,
			rates in rates(),
			on_input in any::<bool>(),
		) {
			let before = reserves(input, output);
			if let Ok(quote) = swap_exact_out(amount_out, before, rates, on_input) {
				prop_assert_eq!(quote.amount_out, amount_out);
				prop_assert!(quote.protocol_fee <= quote.trade_fee);
				// A trade that would push a reserve past `u64::MAX` cannot settle
				// into a real vault, so it must be rejected rather than wrapped.
				match quote.reserves_after(before) {
					Ok(after) => prop_assert!(assert_constant_product(before, after).is_ok()),
					Err(error) => prop_assert_eq!(code(error), AmmError::MathOverflow),
				}
			}
		}

		/// Buying back what an exact-input swap produced never costs less than
		/// that swap paid: quoting is consistent in both directions.
		#[test]
		fn exact_out_never_undercuts_exact_in(
			input in 1_000_000u64..=1u64 << 50,
			output in 1_000_000u64..=1u64 << 50,
			amount_in in 1_000u64..=1u64 << 40,
			rates in rates(),
			on_input in any::<bool>(),
		) {
			let before = reserves(input, output);
			if let Ok(forward) = swap_exact_in(amount_in, before, rates, on_input) {
				let backward = swap_exact_out(forward.amount_out, before, rates, on_input)
					.expect("an output the pool just paid is always quotable");
				prop_assert!(backward.amount_in <= amount_in);
			}
		}

		/// Deposits round up and withdrawals round down, so a round trip never
		/// returns more than it cost.
		#[test]
		fn liquidity_round_trip_never_profits(
			reserve_0 in 1_001u64..=1u64 << 60,
			reserve_1 in 1_001u64..=1u64 << 60,
			supply in 1_001u64..=1u64 << 60,
			lp in 1u64..=1u64 << 40,
		) {
			let deposit = deposit_amounts(lp, supply, reserve_0, reserve_1).expect("deposit");
			let withdraw = withdraw_amounts(
				lp,
				supply + lp,
				reserve_0 + deposit.amount_0,
				reserve_1 + deposit.amount_1,
			);
			if let Ok(withdraw) = withdraw {
				prop_assert!(withdraw.amount_0 <= deposit.amount_0);
				prop_assert!(withdraw.amount_1 <= deposit.amount_1);
			}
		}

		/// The integer square root is always the exact floor.
		#[test]
		fn sqrt_floor_is_exact(value in any::<u128>()) {
			let root = sqrt_floor(value);
			prop_assert!(root * root <= value);
			let next = root + 1;
			prop_assert!(next.checked_mul(next).is_none_or(|square| square > value));
		}
	}
}
