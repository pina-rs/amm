//! `SyncPool`.

use pina::*;

use super::common::PoolSnapshot;
use super::common::store_oracle;
use super::common::unix_now;
use crate::ID;
use crate::events::PoolSynced;
use crate::instructions::SyncPoolInstruction;
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
		// The interval since the last observation is priced with the reserves
		// stored then. The live reserves, including any donation since, only
		// price the interval that starts now.
		let oracle = snapshot.oracle.observe(unix_now()?, reserve_0, reserve_1);
		store_oracle(&mut *self.pool.as_account_mut::<Pool>(&ID)?, oracle);
		PoolSynced::emit(|event| {
			event.pool = *self.pool.address();
			event.reserve_0.set(reserve_0);
			event.reserve_1.set(reserve_1);
			event
				.price_0_cumulative_last
				.set(oracle.price_0_cumulative[0]);
			event
				.price_0_cumulative_high_last
				.set(oracle.price_0_cumulative[1]);
			event.last_update_timestamp.set(oracle.last_update);
			Ok(())
		})
	}
}
