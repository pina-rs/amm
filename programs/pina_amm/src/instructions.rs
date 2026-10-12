//! Instruction discriminators and data layouts.
//!
//! Every instruction's data is its one-byte [`AmmInstruction`] discriminator
//! followed by the packed little-endian fields of its data struct. The account
//! list for each instruction lives next to its handler in
//! [`crate::processors`].

// Pina's schema macros generate zero-copy companion types and accessors without
// doc comments. Every hand-written item and field in this module is documented;
// the allowance only covers the generated companions.
#![allow(missing_docs)]

use pina::*;

use crate::ID;
use crate::errors::AmmError;
use crate::processors::*;
use crate::state::*;

/// Upper bound on the reserved `Migrate` instruction's rent top-up: roughly
/// 6,960 lamports per byte an account grows when its layout changes.
pub const MAX_MIGRATION_LAMPORTS: u64 = 100_000;

/// Instruction discriminators. Values are part of the wire format.
#[discriminator(
	entrypoint,
	migrations(AmmConfig, Pool),
	migrations_max_lamports = MAX_MIGRATION_LAMPORTS
)]
pub enum AmmInstruction {
	/// Create a fee tier. Requires the program's upgrade authority.
	CreateConfig = 0,
	/// Update a fee tier's authority and the rates future pools snapshot.
	UpdateConfig = 1,
	/// Create a pool under a fee tier and make its first deposit.
	CreatePool = 2,
	/// Add liquidity in proportion to the pool's reserves.
	#[dispatch(accounts = LiquidityAccounts)]
	Deposit = 3,
	/// Remove liquidity in proportion to the pool's reserves.
	#[dispatch(accounts = LiquidityAccounts)]
	Withdraw = 4,
	/// Sell an exact amount of one token.
	#[dispatch(accounts = SwapAccounts)]
	SwapExactIn = 5,
	/// Buy an exact amount of one token.
	#[dispatch(accounts = SwapAccounts)]
	SwapExactOut = 6,
	/// Pay a pool's accrued protocol fees to the tier authority's accounts.
	CollectProtocolFees = 7,
	/// Pay a pool's accrued creator fees to the creator's accounts.
	CollectCreatorFees = 8,
	/// Hand a pool's creator-fee rights to another address.
	SetPoolCreator = 9,
	/// Advance a pool's time-weighted price accumulator to now.
	#[dispatch(accounts = SyncPoolAccounts)]
	SyncPool = 10,
}

/// Data for `AmmInstruction::CreateConfig`.
#[instruction(discriminator = AmmInstruction::CreateConfig)]
pub struct CreateConfigInstruction {
	/// Tier index; also the PDA seed.
	pub index: u16,
	/// Swap fee in parts per million.
	pub trade_fee_rate: u32,
	/// Protocol share of the trade fee in parts per million.
	pub protocol_fee_rate: u32,
	/// Creator fee in parts per million.
	pub creator_fee_rate: u32,
	/// Signer that may update the tier and collect its protocol fees.
	pub authority: Address,
	/// The only signer allowed to create pools in this tier, or the default
	/// address to let anyone create pools.
	pub pool_creator_authority: Address,
}

/// Data for `AmmInstruction::UpdateConfig`.
#[instruction(discriminator = AmmInstruction::UpdateConfig)]
pub struct UpdateConfigInstruction {
	/// New tier authority. Pass the current authority to keep it.
	pub new_authority: Address,
	/// Swap fee future pools snapshot, in parts per million.
	pub trade_fee_rate: u32,
	/// Protocol share future pools snapshot, in parts per million.
	pub protocol_fee_rate: u32,
	/// Creator fee future pools snapshot, in parts per million.
	pub creator_fee_rate: u32,
}

/// Data for `AmmInstruction::CreatePool`.
#[instruction(discriminator = AmmInstruction::CreatePool)]
pub struct CreatePoolInstruction {
	/// Initial deposit of token 0.
	#[pina(validate(value != 0, error = AmmError::ZeroAmount))]
	pub amount_0: u64,
	/// Initial deposit of token 1.
	#[pina(validate(value != 0, error = AmmError::ZeroAmount))]
	pub amount_1: u64,
	/// Recipient of the pool's creator fees.
	pub creator: Address,
	/// `CreatorFeeMode` wire value.
	#[pina(validate(value <= 2, error = AmmError::InvalidCreatorFeeMode))]
	pub creator_fee_mode: u8,
}

/// Data for `AmmInstruction::Deposit`.
#[instruction(discriminator = AmmInstruction::Deposit)]
pub struct DepositInstruction {
	/// LP to mint.
	#[pina(validate(value != 0, error = AmmError::ZeroAmount))]
	pub lp_amount: u64,
	/// Most token 0 the depositor will add.
	pub maximum_amount_0: u64,
	/// Most token 1 the depositor will add.
	pub maximum_amount_1: u64,
}

/// Data for `AmmInstruction::Withdraw`.
#[instruction(discriminator = AmmInstruction::Withdraw)]
pub struct WithdrawInstruction {
	/// LP to burn.
	#[pina(validate(value != 0, error = AmmError::ZeroAmount))]
	pub lp_amount: u64,
	/// Least token 0 the owner will accept.
	pub minimum_amount_0: u64,
	/// Least token 1 the owner will accept.
	pub minimum_amount_1: u64,
}

/// Data for `AmmInstruction::SwapExactIn`.
#[instruction(discriminator = AmmInstruction::SwapExactIn)]
pub struct SwapExactInInstruction {
	/// Exact input the trader sells.
	#[pina(validate(value != 0, error = AmmError::ZeroAmount))]
	pub amount_in: u64,
	/// Least output the trader will accept.
	pub minimum_amount_out: u64,
}

/// Data for `AmmInstruction::SwapExactOut`.
#[instruction(discriminator = AmmInstruction::SwapExactOut)]
pub struct SwapExactOutInstruction {
	/// Exact output the trader buys.
	#[pina(validate(value != 0, error = AmmError::ZeroAmount))]
	pub amount_out: u64,
	/// Most input the trader will pay.
	pub maximum_amount_in: u64,
}

/// Data for `AmmInstruction::CollectProtocolFees`.
#[instruction(discriminator = AmmInstruction::CollectProtocolFees)]
pub struct CollectProtocolFeesInstruction {
	/// Most token 0 to collect; `u64::MAX` collects everything accrued.
	pub maximum_amount_0: u64,
	/// Most token 1 to collect; `u64::MAX` collects everything accrued.
	pub maximum_amount_1: u64,
}

/// Data for `AmmInstruction::CollectCreatorFees`.
#[instruction(discriminator = AmmInstruction::CollectCreatorFees)]
pub struct CollectCreatorFeesInstruction {
	/// Most token 0 to collect; `u64::MAX` collects everything accrued.
	pub maximum_amount_0: u64,
	/// Most token 1 to collect; `u64::MAX` collects everything accrued.
	pub maximum_amount_1: u64,
}

/// Data for `AmmInstruction::SyncPool`.
#[instruction(discriminator = AmmInstruction::SyncPool)]
pub struct SyncPoolInstruction {}

/// Data for `AmmInstruction::SetPoolCreator`.
#[instruction(discriminator = AmmInstruction::SetPoolCreator)]
pub struct SetPoolCreatorInstruction {
	/// New recipient of the pool's creator fees.
	pub new_creator: Address,
}
