//! Account layouts, PDA seeds, and fee constants.
//!
//! Pina accounts are packed, fixed width, and decoded at their exact size, so
//! the field order of every account in this module is part of the on-chain
//! ABI. The first byte of each account is its [`AmmAccount`] discriminator and
//! the second is the schema version recorded by `pina migrations`.

// Pina's schema macros generate zero-copy companion types and accessors without
// doc comments. Every hand-written item and field in this module is documented;
// the allowance only covers the generated companions.
#![allow(missing_docs)]

use pina::*;

/// Seed prefix for `AmmConfig` PDAs: `[b"amm_config", index.to_le_bytes()]`.
pub const AMM_CONFIG_SEED: &[u8] = b"amm_config";

/// Seed prefix for `Pool` PDAs: `[b"pool", amm_config, mint_0, mint_1]`.
pub const POOL_SEED: &[u8] = b"pool";

/// Seed prefix for pool vault PDAs: `[b"pool_vault", pool, mint]`.
pub const POOL_VAULT_SEED: &[u8] = b"pool_vault";

/// Seed prefix for pool LP mint PDAs: `[b"pool_lp_mint", pool]`.
pub const POOL_LP_MINT_SEED: &[u8] = b"pool_lp_mint";

/// Denominator for every fee rate: `1_000_000` parts per million.
///
/// A rate of `2_500` is 0.25%.
pub const FEE_RATE_DENOMINATOR: u32 = 1_000_000;

/// Largest combined trade and creator fee rate a tier may charge (10%).
pub const MAX_SWAP_FEE_RATE: u32 = 100_000;

/// LP units counted in a new pool's supply but never minted.
///
/// Locking this liquidity forever means no withdrawal can empty a pool, which
/// removes the first-depositor share-inflation attack.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

/// Decimals of every pool's LP mint.
pub const LP_MINT_DECIMALS: u8 = 9;

/// Account-type discriminators for every account this program owns.
#[discriminator]
pub enum AmmAccount {
	/// A fee tier. See `AmmConfig`.
	AmmConfig = 1,
	/// A liquidity pool. See `Pool`.
	Pool = 2,
}

/// Which token pays a pool's creator fee.
///
/// The trade fee is always taken from the input token. The creator fee can
/// instead be pinned to one side of the pair so a launch creator is paid only
/// in the quote token, whichever direction a trader swaps.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatorFeeMode {
	/// The creator fee is taken from the input token of every swap.
	Input = 0,
	/// The creator fee is always paid in token 0.
	Token0 = 1,
	/// The creator fee is always paid in token 1.
	Token1 = 2,
}

impl CreatorFeeMode {
	/// Decode a stored or submitted mode byte.
	#[must_use]
	pub const fn from_u8(value: u8) -> Option<Self> {
		match value {
			0 => Some(Self::Input),
			1 => Some(Self::Token0),
			2 => Some(Self::Token1),
			_ => None,
		}
	}

	/// Whether the creator fee is taken from the input of a swap that sells
	/// token 0 (`zero_for_one`) or token 1.
	#[must_use]
	pub const fn is_on_input(self, zero_for_one: bool) -> bool {
		match self {
			Self::Input => true,
			Self::Token0 => zero_for_one,
			Self::Token1 => !zero_for_one,
		}
	}
}

/// A fee tier, created by the program's upgrade authority.
///
/// Pools snapshot `trade_fee_rate`, `protocol_fee_rate`, and
/// `creator_fee_rate` when they are created; later updates only affect new
/// pools. `pool_creator_authority` is fixed for the life of the tier.
#[account(discriminator = AmmAccount)]
#[pda(seeds = [AMM_CONFIG_SEED, index: u16], bump = bump)]
pub struct AmmConfig {
	/// Signer that may update this tier and collect protocol fees from every
	/// pool created under it.
	pub authority: Address,
	/// When not the default address, the only signer allowed to create pools
	/// under this tier. The default address leaves pool creation open to
	/// anyone.
	pub pool_creator_authority: Address,
	/// Fee charged on every swap, in parts per million of the fee base. The
	/// LPs keep everything the protocol does not take.
	pub trade_fee_rate: u32,
	/// Share of the trade fee paid to the protocol, in parts per million of
	/// the trade fee.
	pub protocol_fee_rate: u32,
	/// Additional fee paid to the pool's creator, in parts per million of the
	/// fee base.
	pub creator_fee_rate: u32,
	/// Tier index used as the PDA seed.
	pub index: u16,
	/// Canonical bump of this tier's PDA.
	pub bump: u8,
}

/// A constant-product pool for one token pair under one fee tier.
///
/// `mint_0` is always the byte-wise smaller mint address, so every pair has
/// exactly one canonical pool per tier.
#[account(discriminator = AmmAccount)]
#[pda(seeds = [POOL_SEED, amm_config: Address, mint_0: Address, mint_1: Address], bump = bump)]
pub struct Pool {
	/// Fee tier this pool was created under.
	pub amm_config: Address,
	/// Recipient of this pool's creator fees.
	pub creator: Address,
	/// Smaller mint of the pair.
	pub mint_0: Address,
	/// Larger mint of the pair.
	pub mint_1: Address,
	/// Pool-owned token account holding token 0.
	pub vault_0: Address,
	/// Pool-owned token account holding token 1.
	pub vault_1: Address,
	/// LP mint whose mint authority is this pool.
	pub lp_mint: Address,
	/// Economic LP supply, including `MINIMUM_LIQUIDITY` and any LP burned
	/// outside the program. Only deposits and withdrawals change it, so LP
	/// burned directly through the token program stays locked forever.
	pub lp_supply: u64,
	/// Accrued, unpaid protocol fees held in vault 0.
	pub protocol_fees_0: u64,
	/// Accrued, unpaid protocol fees held in vault 1.
	pub protocol_fees_1: u64,
	/// Accrued, unpaid creator fees held in vault 0.
	pub creator_fees_0: u64,
	/// Accrued, unpaid creator fees held in vault 1.
	pub creator_fees_1: u64,
	/// Trade fee rate snapshotted from the tier, in parts per million.
	pub trade_fee_rate: u32,
	/// Protocol share of the trade fee snapshotted from the tier, in parts per
	/// million.
	pub protocol_fee_rate: u32,
	/// Creator fee rate snapshotted from the tier, in parts per million.
	pub creator_fee_rate: u32,
	/// `CreatorFeeMode` wire value.
	pub creator_fee_mode: u8,
	/// Canonical bump of this pool's PDA.
	pub bump: u8,
}

/// Data-free PDA seeds for a pool's vault for one mint.
#[pda(seeds = [POOL_VAULT_SEED, pool: Address, mint: Address])]
pub struct PoolVault {}

/// Data-free PDA seeds for a pool's LP mint.
#[pda(seeds = [POOL_LP_MINT_SEED, pool: Address])]
pub struct PoolLpMint {}
