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

/// Q64.64 price times seconds, stored as low then high `u128` limbs.
pub type PriceCumulative = [u128; 2];

/// Advance the time-weighted price accumulator of token 1 in token 0.
///
/// The accumulator sums `price_0 * elapsed_seconds` as Q64.64 price times
/// seconds, exactly like Uniswap V2: the price in force for an interval is
/// the pool's price at the start of it, so a caller samples the accumulator
/// at two times and divides the difference by the time difference. The
/// reserves must be the ones recorded at `last_update` (see [`PriceOracle`]),
/// never the live vault balances. A pool
/// with an empty side records no price, and a zero `last_update` only
/// anchors the clock, so a first sample after a migration adds nothing.
///
/// The accumulator uses 256 bits. An encoded price is below `2^128` and
/// the total elapsed time in any window is below `2^64`, so its integral
/// is below `2^192`. Subtraction modulo `2^256` therefore recovers the full
/// integral even across a carry out of the low limb, without halting swaps.
#[must_use]
pub fn advance_price_accumulator(
	reserve_0: u64,
	reserve_1: u64,
	cumulative: PriceCumulative,
	last_update: u64,
	now: u64,
) -> (PriceCumulative, u64) {
	if now <= last_update {
		return (cumulative, last_update);
	}
	if last_update == 0 || reserve_0 == 0 || reserve_1 == 0 {
		return (cumulative, now);
	}
	// `reserve_1 < 2^64`, so the Q64.64 numerator always fits in `u128`.
	let price_0 = (u128::from(reserve_1) << 64) / u128::from(reserve_0);
	let elapsed = u128::from(now - last_update);
	// Split the 128-by-64-bit product so no price-time bits are discarded.
	let lower = (price_0 & (Q64 - 1)) * elapsed;
	let upper = (price_0 >> 64) * elapsed;
	let (increment_low, product_carry) = lower.overflowing_add(upper << 64);
	let increment_high = (upper >> 64) + u128::from(product_carry);
	let (low, carry) = cumulative[0].overflowing_add(increment_low);
	let high = cumulative[1]
		.wrapping_add(increment_high)
		.wrapping_add(u128::from(carry));

	([low, high], now)
}

/// A pool's time-weighted price state.
///
/// `reserve_0` and `reserve_1` are the reserves the pool held at the end of
/// the last instruction that observed them, which is the price in force for
/// the whole interval since `last_update`. Reading the reserves the pool
/// holds _now_ for that interval would let a donation followed by any
/// observation reprice time that has already passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceOracle {
	/// Q64.64 price of token 1 in token 0, summed over seconds in 256 bits,
	/// with the low `u128` limb first.
	pub price_0_cumulative: PriceCumulative,
	/// Unix second of the last accumulator advance; zero before the first.
	pub last_update: u64,
	/// Token 0 reserve that prices the interval since `last_update`.
	pub reserve_0: u64,
	/// Token 1 reserve that prices the interval since `last_update`.
	pub reserve_1: u64,
}

impl PriceOracle {
	/// Advance the accumulator to `now` with the stored reserves, then store
	/// `reserve_0` and `reserve_1`, the reserves after the instruction that is
	/// observing the pool, as the price of the next interval.
	///
	/// This is Uniswap V2's `_update`. Every instruction that can change a
	/// pool's reserves calls it once with its post-instruction reserves, so a
	/// value moved within one transaction prices nothing that has already
	/// elapsed, and a price only counts for the time it actually stood.
	#[must_use]
	pub fn observe(self, now: u64, reserve_0: u64, reserve_1: u64) -> Self {
		let (price_0_cumulative, last_update) = advance_price_accumulator(
			self.reserve_0,
			self.reserve_1,
			self.price_0_cumulative,
			self.last_update,
			now,
		);
		Self {
			price_0_cumulative,
			last_update,
			reserve_0,
			reserve_1,
		}
	}
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
	fn exact_out_with_creator_fee_on_input_matches_hand_computation() {
		let quote = swap_exact_out(1_000_000, reserves(10_000_000, 20_000_000), RATES, true)
			.expect("quote");
		// net_in = ceil(10e6 * 1e6 / 19e6) = 526,316.
		// amount_in = ceil(526,316e6 / 997,000) = 527,900, so the combined
		// fee is 1,584; the creator takes floor(1,584 * 500 / 3,000) = 264 of
		// it and the protocol floor(1,320 * 200,000 / 1e6) = 264 of the rest.
		assert_eq!(quote.amount_in, 527_900);
		assert_eq!(quote.amount_out, 1_000_000);
		assert_eq!(quote.trade_fee, 1_320);
		assert_eq!(quote.creator_fee, 264);
		assert_eq!(quote.protocol_fee, 264);
		assert!(quote.creator_fee_on_input);
	}

	#[test]
	fn gross_up_rejects_a_whole_fee_rate() {
		// A rate of 100% leaves nothing net, so gross-up has no answer.
		assert_eq!(
			code(gross_up(1_000, FEE_RATE_DENOMINATOR).expect_err("whole rate must be rejected")),
			AmmError::InvalidFeeRates
		);
	}

	#[test]
	fn the_constant_product_check_rejects_a_decreasing_k() {
		let before = reserves(10_000, 10_000);
		let shrink = reserves(9_999, 10_000);
		let equal = reserves(8_000, 12_500);
		assert_eq!(
			code(
				assert_constant_product(before, shrink)
					.expect_err("a smaller product must be rejected")
			),
			AmmError::InvariantViolation
		);
		assert!(assert_constant_product(before, equal).is_ok());
	}

	#[test]
	fn deposits_of_zero_lp_are_rejected() {
		for (lp_amount, lp_supply) in [(0, 1_000), (1, 0), (0, 0)] {
			assert_eq!(
				code(
					deposit_amounts(lp_amount, lp_supply, 10, 11)
						.expect_err("a zero side must be rejected")
				),
				AmmError::ZeroAmount
			);
		}
	}

	#[test]
	fn withdrawals_can_round_one_side_to_zero_but_not_both() {
		// One raw unit of a 2,000-supply pool against reserves of 1 and
		// 2,000,000 pays nothing of token 0 and 1,000 of token 1; only a
		// withdrawal that pays nothing at all is rejected.
		let amounts = withdraw_amounts(1, 2_000, 1, 2_000_000).expect("withdraw");
		assert_eq!(
			amounts,
			TokenAmounts {
				amount_0: 0,
				amount_1: 1_000,
			}
		);
		assert_eq!(
			code(withdraw_amounts(1, 2_000, 1, 1).expect_err("both sides zero")),
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
		let (cumulative, _) = advance_price_accumulator(100, 200, [0; 2], 1_000, 1_010);
		assert_eq!(cumulative, [price(100, 200) * 10, 0]);
		let (cumulative, _) = advance_price_accumulator(100, 400, cumulative, 1_010, 1_020);
		assert_eq!(cumulative, [price(100, 200) * 10 + price(100, 400) * 10, 0]);
		// A zero start or an empty side anchors the clock without a price.
		assert_eq!(
			advance_price_accumulator(100, 200, [0; 2], 0, 500),
			([0; 2], 500)
		);
		assert_eq!(
			advance_price_accumulator(0, 200, [7, 0], 500, 600),
			([7, 0], 600)
		);
		assert_eq!(
			advance_price_accumulator(100, 0, [7, 0], 500, 600),
			([7, 0], 600)
		);
		// Time moving backwards or standing still changes nothing.
		assert_eq!(
			advance_price_accumulator(100, 200, [7, 0], 600, 599),
			([7, 0], 600)
		);
		assert_eq!(
			advance_price_accumulator(100, 200, [7, 0], 600, 600),
			([7, 0], 600)
		);
	}

	/// Interpret the wire limbs independently of the accumulator arithmetic.
	fn full_width(value: PriceCumulative) -> num_bigint::BigUint {
		(num_bigint::BigUint::from(value[1]) << 128) + num_bigint::BigUint::from(value[0])
	}

	#[test]
	fn the_two_second_extreme_price_does_not_alias_zero() {
		let (cumulative, timestamp) = advance_price_accumulator(1, 1 << 63, [0; 2], 1, 3);
		assert_eq!(timestamp, 3);
		assert_eq!(cumulative, [0, 1]);
		assert_eq!(full_width(cumulative) / 2u8 / Q64, (1u128 << 63).into());
	}

	#[test]
	fn extreme_prices_retain_every_bit_over_the_full_clock_range() {
		for (reserve_0, reserve_1) in [(1, u64::MAX), (u64::MAX, 1), (3, u64::MAX)] {
			for elapsed in [1, 2, 600, u64::MAX - 1] {
				let start = [u128::MAX - 5, 0];
				let (cumulative, timestamp) =
					advance_price_accumulator(reserve_0, reserve_1, start, 1, elapsed + 1);
				let expected = full_width(start)
					+ num_bigint::BigUint::from(q64_price(reserve_0, reserve_1)) * elapsed;

				assert_eq!(timestamp, elapsed + 1);
				assert_eq!(full_width(cumulative), expected);
			}
		}
	}

	#[test]
	fn carries_across_both_limbs_preserve_the_delta() {
		let start = [u128::MAX, u128::MAX];
		let (cumulative, _) = advance_price_accumulator(1, 1, start, 1, 2);
		assert_eq!(cumulative, [Q64 - 1, 0]);
		let modulus = num_bigint::BigUint::from(1u8) << 256;
		assert_eq!(
			full_width(cumulative) + modulus - full_width(start),
			num_bigint::BigUint::from(Q64)
		);
	}

	proptest! {
		#[test]
		fn accumulator_differences_recover_the_full_price_time_integral(
			reserve_0 in 1u64..,
			reserve_1 in 1u64..,
			start in any::<[u128; 2]>(),
			elapsed in 1u64..u64::MAX,
		) {
			let now = 1 + elapsed;
			let (cumulative, timestamp) =
				advance_price_accumulator(reserve_0, reserve_1, start, 1, now);
			let price = (num_bigint::BigUint::from(reserve_1) << 64) / reserve_0;
			let modulus = num_bigint::BigUint::from(1u8) << 256;
			let delta = (full_width(cumulative) + &modulus - full_width(start)) % modulus;

			prop_assert_eq!(timestamp, now);
			prop_assert_eq!(&delta, &(&price * elapsed));
			prop_assert_eq!(delta / elapsed, price);
		}
	}

	fn q64_price(reserve_0: u64, reserve_1: u64) -> u128 {
		(u128::from(reserve_1) << 64) / u128::from(reserve_0)
	}

	/// The audit's attack: reserves of 1e9 and 4e9 stand for ten seconds, then
	/// someone donates token 1 straight to the vault and calls `SyncPool` (or
	/// any swap, deposit, or withdrawal) in the same transaction. The elapsed
	/// ten seconds must still be priced at 4, not at the donated 400.
	#[test]
	fn a_donation_cannot_reprice_the_elapsed_interval() {
		let oracle = PriceOracle {
			price_0_cumulative: [0; 2],
			last_update: 1_000,
			reserve_0: 1_000_000_000,
			reserve_1: 4_000_000_000,
		};
		let honest = oracle.observe(1_010, 1_000_000_000, 4_000_000_000);
		let donated = oracle.observe(1_010, 1_000_000_000, 400_000_000_000);
		assert_eq!(
			honest.price_0_cumulative,
			[q64_price(1_000_000_000, 4_000_000_000) * 10, 0]
		);
		assert_eq!(donated.price_0_cumulative, honest.price_0_cumulative);
		assert_eq!(donated.last_update, 1_010);
		// The donated price counts only for the time it actually stands.
		let later = donated.observe(1_011, 1_000_000_000, 4_000_000_000);
		assert_eq!(
			full_width(later.price_0_cumulative),
			full_width(honest.price_0_cumulative) + q64_price(1_000_000_000, 400_000_000_000)
		);
	}

	/// Moving the price and restoring it within one second accrues nothing:
	/// only the reserves left standing when the clock ticks are priced.
	#[test]
	fn same_second_observations_only_keep_the_last_reserves() {
		let oracle = PriceOracle {
			price_0_cumulative: [7, 0],
			last_update: 1_000,
			reserve_0: 100,
			reserve_1: 200,
		};
		let moved = oracle.observe(1_000, 100, 200_000);
		assert_eq!(moved.price_0_cumulative, [7, 0]);
		let restored = moved.observe(1_000, 100, 200);
		assert_eq!(restored, oracle);
	}

	/// One instruction the property test applies to a model pool.
	#[derive(Debug, Clone)]
	enum Operation {
		SwapExactIn {
			zero_for_one: bool,
			amount: u64,
		},
		SwapExactOut {
			zero_for_one: bool,
			amount: u64,
		},
		Deposit {
			lp: u64,
		},
		Withdraw {
			lp: u64,
		},
		/// A plain token transfer into a vault: no program call, no observation.
		Donate {
			to_0: bool,
			amount: u64,
		},
		Sync,
	}

	fn operation() -> impl Strategy<Value = Operation> {
		prop_oneof![
			(any::<bool>(), 1u64..1_000_000_000).prop_map(|(zero_for_one, amount)| {
				Operation::SwapExactIn {
					zero_for_one,
					amount,
				}
			}),
			(any::<bool>(), 1u64..100_000_000).prop_map(|(zero_for_one, amount)| {
				Operation::SwapExactOut {
					zero_for_one,
					amount,
				}
			}),
			(1u64..1_000_000_000).prop_map(|lp| Operation::Deposit { lp }),
			(1u64..1_000_000_000).prop_map(|lp| Operation::Withdraw { lp }),
			(any::<bool>(), 1u64..1_000_000_000_000)
				.prop_map(|(to_0, amount)| Operation::Donate { to_0, amount }),
			Just(Operation::Sync),
		]
	}

	/// Live reserves after `operation`, or `None` when the instruction would
	/// fail and roll back. Donations return the new reserves but are not
	/// program instructions, so they never observe the oracle.
	fn apply(
		operation: &Operation,
		reserve_0: u64,
		reserve_1: u64,
		lp_supply: u64,
	) -> Option<(u64, u64, u64)> {
		let orient = |zero_for_one: bool| {
			if zero_for_one {
				reserves(reserve_0, reserve_1)
			} else {
				reserves(reserve_1, reserve_0)
			}
		};
		let settle = |zero_for_one: bool, quote: SwapQuote| {
			let after = quote.reserves_after(orient(zero_for_one)).ok()?;
			Some(if zero_for_one {
				(after.input, after.output, lp_supply)
			} else {
				(after.output, after.input, lp_supply)
			})
		};
		match *operation {
			Operation::SwapExactIn {
				zero_for_one,
				amount,
			} => {
				settle(
					zero_for_one,
					swap_exact_in(amount, orient(zero_for_one), RATES, true).ok()?,
				)
			}
			Operation::SwapExactOut {
				zero_for_one,
				amount,
			} => {
				settle(
					zero_for_one,
					swap_exact_out(amount, orient(zero_for_one), RATES, false).ok()?,
				)
			}
			Operation::Deposit { lp } => {
				let amounts = deposit_amounts(lp, lp_supply, reserve_0, reserve_1).ok()?;
				Some((
					reserve_0.checked_add(amounts.amount_0)?,
					reserve_1.checked_add(amounts.amount_1)?,
					lp_supply.checked_add(lp)?,
				))
			}
			Operation::Withdraw { lp } => {
				let amounts = withdraw_amounts(lp, lp_supply, reserve_0, reserve_1).ok()?;
				Some((
					reserve_0 - amounts.amount_0,
					reserve_1 - amounts.amount_1,
					lp_supply - lp,
				))
			}
			Operation::Donate { to_0, amount } => {
				if to_0 {
					Some((reserve_0.checked_add(amount)?, reserve_1, lp_supply))
				} else {
					Some((reserve_0, reserve_1.checked_add(amount)?, lp_supply))
				}
			}
			Operation::Sync => Some((reserve_0, reserve_1, lp_supply)),
		}
	}

	proptest! {
		/// Across any mix of swaps, deposits, withdrawals, donations, and
		/// syncs, the accumulator equals the integral of the price the pool
		/// held at each whole second, rebuilt second by second from the
		/// observations alone. A donation only ever enters that timeline at
		/// the next instruction's own timestamp, never earlier.
		#[test]
		fn accrual_prices_each_second_with_the_reserves_standing_then(
			steps in proptest::collection::vec((0u64..120, operation()), 1..40),
		) {
			let start = 1_000u64;
			let (mut reserve_0, mut reserve_1) = (1_000_000_000u64, 4_000_000_000u64);
			let mut lp_supply = initial_liquidity(reserve_0, reserve_1).unwrap().lp_supply;
			let mut oracle = PriceOracle {
				price_0_cumulative: [0; 2],
				last_update: start,
				reserve_0,
				reserve_1,
			};
			// Every observation in order: (timestamp, reserve_0, reserve_1).
			let mut timeline = std::vec![(start, reserve_0, reserve_1)];
			let mut now = start;
			for (elapsed, operation) in &steps {
				now += elapsed;
				let Some((next_0, next_1, next_supply)) =
					apply(operation, reserve_0, reserve_1, lp_supply)
				else {
					continue;
				};
				(reserve_0, reserve_1, lp_supply) = (next_0, next_1, next_supply);
				if matches!(operation, Operation::Donate { .. }) {
					continue;
				}
				oracle = oracle.observe(now, reserve_0, reserve_1);
				prop_assert_eq!((oracle.reserve_0, oracle.reserve_1), (reserve_0, reserve_1));
				prop_assert_eq!(oracle.last_update, now);
				timeline.push((now, reserve_0, reserve_1));
			}

			// Each second from `start` up to the last observation is priced by
			// the latest observation made at or before it.
			let end = oracle.last_update;
			let mut expected = num_bigint::BigUint::from(0u8);
			for second in start..end {
				let &(_, standing_0, standing_1) = timeline
					.iter()
					.rev()
					.find(|(at, _, _)| *at <= second)
					.unwrap();
				expected += q64_price(standing_0, standing_1);
			}
			prop_assert_eq!(full_width(oracle.price_0_cumulative), expected);
		}

		/// Whatever the live reserves are when an instruction observes the
		/// pool, the interval it closes is priced only by the stored ones.
		#[test]
		fn the_closed_interval_ignores_the_new_reserves(
			stored_0 in 1u64..,
			stored_1 in 1u64..,
			start in any::<[u128; 2]>(),
			last_update in 1u64..u64::MAX / 2,
			elapsed in 0u64..1_000_000,
			live_a in (any::<u64>(), any::<u64>()),
			live_b in (any::<u64>(), any::<u64>()),
		) {
			let oracle = PriceOracle {
				price_0_cumulative: start,
				last_update,
				reserve_0: stored_0,
				reserve_1: stored_1,
			};
			let now = last_update + elapsed;
			let a = oracle.observe(now, live_a.0, live_a.1);
			let b = oracle.observe(now, live_b.0, live_b.1);
			prop_assert_eq!(a.price_0_cumulative, b.price_0_cumulative);
			let modulus = num_bigint::BigUint::from(1u8) << 256;
			let delta = (full_width(a.price_0_cumulative) + &modulus - full_width(start)) % modulus;
			prop_assert_eq!(delta, num_bigint::BigUint::from(q64_price(stored_0, stored_1)) * elapsed);
		}
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

		/// Every rounded quantity in a quote is the exact floor or ceiling of
		/// its ideal rational value, verified by cross-multiplication:
		/// `floor(n/d)` is exactly `q` when `q*d <= n < (q+1)*d`, and
		/// `ceil(n/d)` is exactly `q` when `q*d - d < n <= q*d`. The checks
		/// divide nothing, so they share no arithmetic with the
		/// implementation they judge.
		#[test]
		fn quotes_round_exactly_against_their_ideal_values(
			input in 1_000u64..=u64::MAX / 4,
			output in 1_000u64..=u64::MAX / 4,
			amount in 1_000u64..=u64::MAX / 4,
			rates in rates(),
			on_input in any::<bool>(),
		) {
			let before = reserves(input, output);
			let exact_ceiling = |q: u128, n: u128, d: u128| q * d >= n && q * d - n < d;
			let exact_floor = |q: u128, n: u128, d: u128| q * d <= n && n - q * d < d;
			let d = DENOMINATOR;

			if let Ok(quote) = swap_exact_in(amount, before, rates, on_input) {
				let protocol = u128::from(quote.trade_fee) * u128::from(rates.protocol);
				prop_assert!(exact_floor(u128::from(quote.protocol_fee), protocol, d));
				if on_input {
					let total = u128::from(quote.trade_fee + quote.creator_fee);
					prop_assert!(exact_ceiling(
						total,
						u128::from(amount) * u128::from(rates.trade + rates.creator),
						d
					));
					let combined = u128::from(rates.trade) + u128::from(rates.creator);
					prop_assert!(exact_floor(
						u128::from(quote.creator_fee),
						total * u128::from(rates.creator),
						combined
					));
					let net = u128::from(amount - quote.trade_fee - quote.creator_fee);
					prop_assert!(exact_floor(
						u128::from(quote.amount_out),
						net * u128::from(output),
						net + u128::from(input)
					));
				} else {
					prop_assert!(exact_ceiling(
						u128::from(quote.trade_fee),
						u128::from(amount) * u128::from(rates.trade),
						d
					));
					let net = u128::from(amount - quote.trade_fee);
					let gross = u128::from(quote.amount_out + quote.creator_fee);
					prop_assert!(exact_floor(
						gross,
						net * u128::from(output),
						net + u128::from(input)
					));
					prop_assert!(exact_ceiling(
						u128::from(quote.creator_fee),
						gross * u128::from(rates.creator),
						d
					));
				}
			}

			if let Ok(quote) = swap_exact_out(amount, before, rates, on_input) {
				prop_assert_eq!(quote.amount_out, amount);
				let protocol = u128::from(quote.trade_fee) * u128::from(rates.protocol);
				prop_assert!(exact_floor(u128::from(quote.protocol_fee), protocol, d));
				if on_input {
					let total = u128::from(quote.trade_fee + quote.creator_fee);
					let net_in = u128::from(quote.amount_in) - total;
					prop_assert!(exact_ceiling(
						net_in,
						u128::from(input) * u128::from(amount),
						u128::from(output - amount)
					));
					let combined = u128::from(rates.trade + rates.creator);
					prop_assert!(exact_ceiling(
						u128::from(quote.amount_in),
						net_in * d,
						d - combined
					));
					prop_assert!(exact_floor(
						u128::from(quote.creator_fee),
						total * u128::from(rates.creator),
						combined
					));
				} else {
					let gross_out = u128::from(quote.amount_out + quote.creator_fee);
					let net_in = u128::from(quote.amount_in) - u128::from(quote.trade_fee);
					prop_assert!(exact_ceiling(
						gross_out,
						u128::from(amount) * d,
						d - u128::from(rates.creator)
					));
					prop_assert!(exact_ceiling(
						net_in,
						u128::from(input) * gross_out,
						u128::from(output) - gross_out
					));
					prop_assert!(exact_ceiling(
						u128::from(quote.amount_in),
						net_in * d,
						d - u128::from(rates.trade)
					));
				}
			}
		}

		/// Deposit and withdrawal amounts are the exact ceiling and floor of
		/// the caller's rational share of each reserve.
		#[test]
		fn liquidity_amounts_round_exactly_against_their_ideal_shares(
			reserve_0 in 1_001u64..=1u64 << 60,
			reserve_1 in 1_001u64..=1u64 << 60,
			supply in 1_001u64..=1u64 << 60,
			lp in 1u64..=1u64 << 40,
		) {
			let exact_ceiling = |q: u128, n: u128, d: u128| q * d >= n && q * d - n < d;
			let exact_floor = |q: u128, n: u128, d: u128| q * d <= n && n - q * d < d;
			let deposit = deposit_amounts(lp, supply, reserve_0, reserve_1).expect("deposit");
			prop_assert!(exact_ceiling(
				u128::from(deposit.amount_0),
				u128::from(reserve_0) * u128::from(lp),
				u128::from(supply)
			));
			prop_assert!(exact_ceiling(
				u128::from(deposit.amount_1),
				u128::from(reserve_1) * u128::from(lp),
				u128::from(supply)
			));
			if let Ok(withdraw) = withdraw_amounts(lp, supply, reserve_0, reserve_1) {
				prop_assert!(exact_floor(
					u128::from(withdraw.amount_0),
					u128::from(reserve_0) * u128::from(lp),
					u128::from(supply)
				));
				prop_assert!(exact_floor(
					u128::from(withdraw.amount_1),
					u128::from(reserve_1) * u128::from(lp),
					u128::from(supply)
				));
			}
		}
	}
}
