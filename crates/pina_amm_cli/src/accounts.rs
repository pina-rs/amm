//! Address derivation and account decoding.

use pina_amm_client::PINA_AMM_ID;
use pina_amm_client::accounts::AmmConfig;
use pina_amm_client::accounts::Pool;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::context::Context;
use crate::error::CliError;

/// The original SPL Token program, which owns every LP mint.
pub const TOKEN_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// The associated token account program.
pub const ATA_PROGRAM: Pubkey =
	Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
/// The system program.
pub const SYSTEM_PROGRAM: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
/// The upgradeable BPF loader.
pub const UPGRADEABLE_LOADER: Pubkey =
	Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");

/// Byte offset of the `amount` field in an SPL Token or Token-2022 account.
const TOKEN_AMOUNT_OFFSET: usize = 64;

/// The tier PDA for `index`.
pub fn config_address(index: u16) -> Pubkey {
	AmmConfig::find_pda(index).0
}

/// The AMM's program-data account, which records its upgrade authority.
pub fn program_data_address() -> Pubkey {
	Pubkey::find_program_address(&[PINA_AMM_ID.as_ref()], &UPGRADEABLE_LOADER).0
}

/// A pool's vault for `mint`.
pub fn vault_address(pool: &Pubkey, mint: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(&[b"pool_vault", pool.as_ref(), mint.as_ref()], &PINA_AMM_ID).0
}

/// A pool's LP mint.
pub fn lp_mint_address(pool: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(&[b"pool_lp_mint", pool.as_ref()], &PINA_AMM_ID).0
}

/// `owner`'s associated token account for `mint`.
pub fn associated_token_address(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
		&ATA_PROGRAM,
	)
	.0
}

/// Create `owner`'s associated token account for `mint` unless it exists.
pub fn create_associated_token_account(
	payer: &Pubkey,
	owner: &Pubkey,
	mint: &Pubkey,
	token_program: &Pubkey,
) -> Instruction {
	Instruction::new_with_bytes(
		ATA_PROGRAM,
		&[1],
		vec![
			AccountMeta::new(*payer, true),
			AccountMeta::new(associated_token_address(owner, mint, token_program), false),
			AccountMeta::new_readonly(*owner, false),
			AccountMeta::new_readonly(*mint, false),
			AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
			AccountMeta::new_readonly(*token_program, false),
		],
	)
}

/// Sort two mints into the pool's `(mint_0, mint_1)` order.
pub fn ordered(mint_a: Pubkey, mint_b: Pubkey) -> Result<(Pubkey, Pubkey), CliError> {
	match mint_a.cmp(&mint_b) {
		std::cmp::Ordering::Less => Ok((mint_a, mint_b)),
		std::cmp::Ordering::Greater => Ok((mint_b, mint_a)),
		std::cmp::Ordering::Equal => Err(CliError::SameMint),
	}
}

/// The token program that owns `mint`.
pub fn token_program_of(context: &Context, mint: &Pubkey) -> Result<Pubkey, CliError> {
	Ok(context.account(mint)?.owner)
}

/// A decoded tier.
pub struct ConfigView {
	/// The tier's address.
	pub address: Pubkey,
	/// Signer that updates the tier and collects its protocol fees.
	pub authority: Pubkey,
	/// The only signer allowed to create pools, or the default address.
	pub pool_creator_authority: Pubkey,
	/// Trade fee in parts per million.
	pub trade_fee_rate: u32,
	/// Protocol share of the trade fee in parts per million.
	pub protocol_fee_rate: u32,
	/// Creator fee in parts per million.
	pub creator_fee_rate: u32,
	/// Tier index.
	pub index: u16,
}

/// Fetch and decode a tier, checking that the AMM owns it.
pub fn load_config(context: &Context, address: &Pubkey) -> Result<ConfigView, CliError> {
	let account = context.account(address)?;
	require_owner(address, &account.owner)?;
	let config =
		AmmConfig::from_bytes(&account.data).map_err(|_| CliError::InvalidAccountData(*address))?;
	Ok(ConfigView {
		address: *address,
		authority: config.authority,
		pool_creator_authority: config.pool_creator_authority,
		trade_fee_rate: config.trade_fee_rate.get(),
		protocol_fee_rate: config.protocol_fee_rate.get(),
		creator_fee_rate: config.creator_fee_rate.get(),
		index: config.index.get(),
	})
}

/// A decoded pool together with its live reserves.
pub struct PoolView {
	/// The pool's address.
	pub address: Pubkey,
	/// Fee tier.
	pub amm_config: Pubkey,
	/// Recipient of creator fees.
	pub creator: Pubkey,
	/// Smaller mint.
	pub mint_0: Pubkey,
	/// Larger mint.
	pub mint_1: Pubkey,
	/// Token program of `mint_0`.
	pub token_program_0: Pubkey,
	/// Token program of `mint_1`.
	pub token_program_1: Pubkey,
	/// Vault for `mint_0`.
	pub vault_0: Pubkey,
	/// Vault for `mint_1`.
	pub vault_1: Pubkey,
	/// LP mint.
	pub lp_mint: Pubkey,
	/// Economic LP supply.
	pub lp_supply: u64,
	/// Liquidity reserve of `mint_0`, excluding accrued fees.
	pub reserve_0: u64,
	/// Liquidity reserve of `mint_1`, excluding accrued fees.
	pub reserve_1: u64,
	/// Accrued protocol fees, per token.
	pub protocol_fees: (u64, u64),
	/// Accrued creator fees, per token.
	pub creator_fees: (u64, u64),
	/// Trade fee rate in parts per million.
	pub trade_fee_rate: u32,
	/// Protocol share in parts per million.
	pub protocol_fee_rate: u32,
	/// Creator fee rate in parts per million.
	pub creator_fee_rate: u32,
	/// Creator fee mode wire value.
	pub creator_fee_mode: u8,
}

impl PoolView {
	/// The `(input mint, output mint)` side flags for selling `mint`.
	pub fn sells_token_0(&self, mint: &Pubkey) -> Result<bool, CliError> {
		if mint == &self.mint_0 {
			Ok(true)
		} else if mint == &self.mint_1 {
			Ok(false)
		} else {
			Err(CliError::MintNotInPool {
				mint: *mint,
				pool: self.address,
			})
		}
	}
}

/// Fetch and decode a pool and its vault balances.
pub fn load_pool(context: &Context, address: &Pubkey) -> Result<PoolView, CliError> {
	let account = context.account(address)?;
	require_owner(address, &account.owner)?;
	let pool =
		Pool::from_bytes(&account.data).map_err(|_| CliError::InvalidAccountData(*address))?;
	let protocol_fees = (pool.protocol_fees0.get(), pool.protocol_fees1.get());
	let creator_fees = (pool.creator_fees0.get(), pool.creator_fees1.get());
	let vault_0 = context.account(&pool.vault0)?;
	let vault_1 = context.account(&pool.vault1)?;
	let balance = |data: &[u8], address: &Pubkey| {
		data.get(TOKEN_AMOUNT_OFFSET..TOKEN_AMOUNT_OFFSET + 8)
			.and_then(|bytes| bytes.try_into().ok())
			.map(u64::from_le_bytes)
			.ok_or(CliError::InvalidAccountData(*address))
	};
	let vault_0_balance = balance(&vault_0.data, &pool.vault0)?;
	let vault_1_balance = balance(&vault_1.data, &pool.vault1)?;
	Ok(PoolView {
		address: *address,
		amm_config: pool.amm_config,
		creator: pool.creator,
		mint_0: pool.mint0,
		mint_1: pool.mint1,
		token_program_0: vault_0.owner,
		token_program_1: vault_1.owner,
		vault_0: pool.vault0,
		vault_1: pool.vault1,
		lp_mint: pool.lp_mint,
		lp_supply: pool.lp_supply.get(),
		reserve_0: vault_0_balance.saturating_sub(protocol_fees.0 + creator_fees.0),
		reserve_1: vault_1_balance.saturating_sub(protocol_fees.1 + creator_fees.1),
		protocol_fees,
		creator_fees,
		trade_fee_rate: pool.trade_fee_rate.get(),
		protocol_fee_rate: pool.protocol_fee_rate.get(),
		creator_fee_rate: pool.creator_fee_rate.get(),
		creator_fee_mode: pool.creator_fee_mode,
	})
}

fn require_owner(address: &Pubkey, owner: &Pubkey) -> Result<(), CliError> {
	if owner == &PINA_AMM_ID {
		return Ok(());
	}
	Err(CliError::WrongOwner {
		address: *address,
		owner: *owner,
		expected: PINA_AMM_ID,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn orders_mints_and_rejects_duplicates() {
		let low = Pubkey::new_from_array([1; 32]);
		let high = Pubkey::new_from_array([2; 32]);
		assert_eq!(ordered(high, low).expect("ordered"), (low, high));
		assert_eq!(ordered(low, high).expect("ordered"), (low, high));
		assert!(matches!(ordered(low, low), Err(CliError::SameMint)));
	}

	#[test]
	fn derives_the_same_pool_as_the_client() {
		let config = config_address(0);
		let (mint_0, mint_1) =
			ordered(Pubkey::new_unique(), Pubkey::new_unique()).expect("ordered");
		let pool = Pool::find_pda(&config, &mint_0, &mint_1).0;
		assert_ne!(vault_address(&pool, &mint_0), vault_address(&pool, &mint_1));
		assert_ne!(lp_mint_address(&pool), pool);
	}
}
