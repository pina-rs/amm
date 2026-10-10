//! `SyncPool`.

use pina::sysvars::Sysvar;
use pina::sysvars::clock::Clock;
use pina::*;

use super::common::PoolSnapshot;
use crate::ID;
use crate::errors::AmmError;
use crate::events::PoolSynced;
use crate::instructions::SyncPoolInstruction;
use crate::math::advance_price_accumulator;
use crate::state::Pool;

/// Accounts for `SyncPool`.
#[derive(Accounts, Debug)]
pub struct SyncPoolAccounts<'a> {
	/// The pool whose accumulator is advanced.
	pub pool: &'a mut AccountView,
	/// The pool's token 0 vault.
	pub vault_0: &'a AccountView,
	/// The pool's token 1 vault.
	pub vault_1: &'a AccountView,
	/// Token program that owns token 0.
	pub token_program_0: &'a AccountView,
	/// Token program that owns token 1.
	pub token_program_1: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for SyncPoolAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		SyncPoolInstruction::try_from_bytes(data)?;
		let snapshot = PoolSnapshot::load(self.pool)?;
		snapshot.assert_vaults(self.vault_0, self.vault_1)?;
		let (reserve_0, reserve_1) = snapshot.reserves(
			self.vault_0,
			self.vault_1,
			self.token_program_0,
			self.token_program_1,
		)?;
		let now =
			u64::try_from(Clock::get()?.unix_timestamp).map_err(|_| AmmError::MathOverflow)?;
		let (price_0_cumulative_last, last_update_timestamp) = advance_price_accumulator(
			reserve_0,
			reserve_1,
			snapshot.price_0_cumulative_last,
			snapshot.last_update_timestamp,
			now,
		)?;
		{
			let mut pool = self.pool.as_account_mut::<Pool>(&ID)?;
			pool.price_0_cumulative_last.set(price_0_cumulative_last);
			pool.last_update_timestamp.set(last_update_timestamp);
		}
		PoolSynced::emit(|event| {
			event.pool = *self.pool.address();
			event.reserve_0.set(reserve_0);
			event.reserve_1.set(reserve_1);
			event.price_0_cumulative_last.set(price_0_cumulative_last);
			event.last_update_timestamp.set(last_update_timestamp);
			Ok(())
		})
	}
}
