//! Pool loading and reserve accounting shared by the processors.

use pina::*;

use crate::ID;
use crate::errors::AmmError;
use crate::math::FeeRates;
use crate::state::CreatorFeeMode;
use crate::state::Pool;
use crate::token::vault_amount;

/// A copy of a pool's state, taken so the account borrow can be released
/// before any CPI touches the pool.
#[derive(Clone, Copy)]
pub(crate) struct PoolSnapshot {
	pub amm_config: Address,
	pub creator: Address,
	pub mint_0: Address,
	pub mint_1: Address,
	pub vault_0: Address,
	pub vault_1: Address,
	pub lp_mint: Address,
	pub lp_supply: u64,
	pub protocol_fees_0: u64,
	pub protocol_fees_1: u64,
	pub creator_fees_0: u64,
	pub creator_fees_1: u64,
	pub rates: FeeRates,
	pub creator_fee_mode: CreatorFeeMode,
	pub bump: u8,
}

impl PoolSnapshot {
	/// Load a pool. Ownership, discriminator, schema version, and size are
	/// checked by the typed load; only this program creates accounts that pass
	/// it, and it only creates pools at their canonical PDA.
	pub fn load(pool: &AccountView) -> Result<Self, ProgramError> {
		let state = pool.as_account::<Pool>(&ID)?;
		let creator_fee_mode = CreatorFeeMode::from_u8(state.creator_fee_mode)
			.ok_or(AmmError::InvalidCreatorFeeMode)?;
		Ok(Self {
			amm_config: state.amm_config,
			creator: state.creator,
			mint_0: state.mint_0,
			mint_1: state.mint_1,
			vault_0: state.vault_0,
			vault_1: state.vault_1,
			lp_mint: state.lp_mint,
			lp_supply: state.lp_supply.get(),
			protocol_fees_0: state.protocol_fees_0.get(),
			protocol_fees_1: state.protocol_fees_1.get(),
			creator_fees_0: state.creator_fees_0.get(),
			creator_fees_1: state.creator_fees_1.get(),
			rates: FeeRates {
				trade: state.trade_fee_rate.get(),
				protocol: state.protocol_fee_rate.get(),
				creator: state.creator_fee_rate.get(),
			},
			creator_fee_mode,
			bump: state.bump,
		})
	}

	/// Require `vault_0` and `vault_1` to be this pool's vaults, in order.
	pub fn assert_vaults(&self, vault_0: &AccountView, vault_1: &AccountView) -> ProgramResult {
		if vault_0.address() != &self.vault_0 || vault_1.address() != &self.vault_1 {
			return Err(AmmError::PoolAccountMismatch.into());
		}
		Ok(())
	}

	/// Liquidity reserves: vault balances minus fees accrued but not yet paid.
	pub fn reserves(
		&self,
		vault_0: &AccountView,
		vault_1: &AccountView,
		token_program_0: &AccountView,
		token_program_1: &AccountView,
	) -> Result<(u64, u64), ProgramError> {
		let reserve_0 = vault_amount(vault_0, token_program_0)?
			.checked_sub(self.protocol_fees_0)
			.and_then(|value| value.checked_sub(self.creator_fees_0))
			.ok_or(AmmError::VaultAccountingMismatch)?;
		let reserve_1 = vault_amount(vault_1, token_program_1)?
			.checked_sub(self.protocol_fees_1)
			.and_then(|value| value.checked_sub(self.creator_fees_1))
			.ok_or(AmmError::VaultAccountingMismatch)?;
		Ok((reserve_0, reserve_1))
	}
}

/// Run `body` with the pool's PDA signer.
pub(crate) fn with_pool_signer<R>(
	snapshot: &PoolSnapshot,
	body: impl FnOnce(&Signer<'_, '_>) -> Result<R, ProgramError>,
) -> Result<R, ProgramError> {
	let seeds = Pool::seeds(&snapshot.amm_config, &snapshot.mint_0, &snapshot.mint_1)
		.with_bump(snapshot.bump);
	let signer = seeds.to_signer();
	body(&signer.as_signer())
}
