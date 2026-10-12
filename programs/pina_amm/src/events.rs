//! Events written to the transaction log.
//!
//! Each event is emitted with Pina's `emit` helper as a `Program data:` log
//! line. Generated clients decode them with `parsePinaAmmEventsFromLogs`
//! (TypeScript and Dart) or each event's `try_from_bytes` (Rust). Always pass
//! the complete, ordered logs of one transaction so events are attributed to
//! the program that actually emitted them.

// Pina's schema macros generate zero-copy companion types and accessors without
// doc comments. Every hand-written item and field in this module is documented;
// the allowance only covers the generated companions.
#![allow(missing_docs)]

use pina::*;

/// Event discriminators. Values are part of the wire format.
#[discriminator]
pub enum AmmEvent {
	/// A pool was created. See `PoolCreated`.
	PoolCreated = 1,
	/// A swap settled. See `Swapped`.
	Swapped = 2,
	/// Liquidity was added or removed. See `LiquidityChanged`.
	LiquidityChanged = 3,
	/// Accrued fees were paid out. See `FeesCollected`.
	FeesCollected = 4,
	/// A pool's price accumulator advanced. See `PoolSynced`.
	PoolSynced = 5,
	/// A fee tier was updated. See `ConfigUpdated`.
	ConfigUpdated = 6,
	/// A pool's creator-fee rights moved. See `PoolCreatorChanged`.
	PoolCreatorChanged = 7,
}

/// Emitted by `CreatePool`.
#[event(discriminator = AmmEvent)]
pub struct PoolCreated {
	/// The new pool.
	pub pool: Address,
	/// Fee tier the pool was created under.
	pub amm_config: Address,
	/// Recipient of the pool's creator fees.
	pub creator: Address,
	/// Smaller mint of the pair.
	pub mint_0: Address,
	/// Larger mint of the pair.
	pub mint_1: Address,
	/// The pool's LP mint.
	pub lp_mint: Address,
	/// Initial reserve of token 0.
	pub amount_0: u64,
	/// Initial reserve of token 1.
	pub amount_1: u64,
	/// Initial economic LP supply, including the locked minimum.
	pub lp_supply: u64,
}

/// Emitted by `SwapExactIn` and `SwapExactOut`.
#[event(discriminator = AmmEvent)]
pub struct Swapped {
	/// The pool traded against.
	pub pool: Address,
	/// The trader who signed.
	pub trader: Address,
	/// `1` when the trader sold token 0 for token 1, `0` for the reverse.
	pub zero_for_one: u8,
	/// Tokens the trader paid.
	pub amount_in: u64,
	/// Tokens the trader received.
	pub amount_out: u64,
	/// Trade fee in the input token, including the protocol share.
	pub trade_fee: u64,
	/// Protocol share of the trade fee, in the input token.
	pub protocol_fee: u64,
	/// Creator fee. It is in the input token when `creator_fee_on_input` is
	/// set and in the output token otherwise.
	pub creator_fee: u64,
	/// `1` when `creator_fee` is denominated in the input token, `0` when it is
	/// in the output token.
	pub creator_fee_on_input: u8,
	/// Token 0 reserve after the swap.
	pub reserve_0: u64,
	/// Token 1 reserve after the swap.
	pub reserve_1: u64,
}

/// Emitted by `Deposit` and `Withdraw`.
#[event(discriminator = AmmEvent)]
pub struct LiquidityChanged {
	/// The pool whose liquidity changed.
	pub pool: Address,
	/// The LP owner who signed.
	pub owner: Address,
	/// `1` for a deposit and `0` for a withdrawal.
	pub is_deposit: u8,
	/// LP minted or burned.
	pub lp_amount: u64,
	/// Token 0 added or removed.
	pub amount_0: u64,
	/// Token 1 added or removed.
	pub amount_1: u64,
	/// Economic LP supply after the change.
	pub lp_supply: u64,
}

/// Emitted by `CollectProtocolFees` and `CollectCreatorFees`.
#[event(discriminator = AmmEvent)]
pub struct FeesCollected {
	/// The pool the fees were accrued in.
	pub pool: Address,
	/// The signer who collected them.
	pub collector: Address,
	/// `1` for protocol fees and `0` for creator fees.
	pub is_protocol: u8,
	/// Token 0 paid out.
	pub amount_0: u64,
	/// Token 1 paid out.
	pub amount_1: u64,
}

/// Emitted by `SyncPool`. Swaps advance the same accumulator and report
/// their reserves through `Swapped`.
#[event(discriminator = AmmEvent)]
pub struct PoolSynced {
	/// The pool whose accumulator advanced.
	pub pool: Address,
	/// Token 0 reserve at the advance.
	pub reserve_0: u64,
	/// Token 1 reserve at the advance.
	pub reserve_1: u64,
	/// The accumulator after the advance: Q64.64 price of token 1 in token 0,
	/// summed over seconds.
	pub price_0_cumulative_last: u128,
	/// Unix second of the advance.
	pub last_update_timestamp: u64,
}

/// Emitted by `UpdateConfig`.
#[event(discriminator = AmmEvent)]
pub struct ConfigUpdated {
	/// The tier that was updated.
	pub amm_config: Address,
	/// The tier's new authority.
	pub new_authority: Address,
	/// Trade fee future pools snapshot, in parts per million.
	pub trade_fee_rate: u32,
	/// Protocol share future pools snapshot, in parts per million.
	pub protocol_fee_rate: u32,
	/// Creator fee future pools snapshot, in parts per million.
	pub creator_fee_rate: u32,
}

/// Emitted by `SetPoolCreator`.
#[event(discriminator = AmmEvent)]
pub struct PoolCreatorChanged {
	/// The pool whose creator-fee rights moved.
	pub pool: Address,
	/// The previous creator.
	pub previous_creator: Address,
	/// The new creator.
	pub new_creator: Address,
}
