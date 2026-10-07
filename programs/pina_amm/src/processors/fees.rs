//! `CollectProtocolFees`, `CollectCreatorFees`, and `SetPoolCreator`.

use pina::*;

use super::common::PoolSnapshot;
use super::common::with_pool_signer;
use crate::ID;
use crate::errors::AmmError;
use crate::events::FeesCollected;
use crate::instructions::CollectCreatorFeesInstruction;
use crate::instructions::CollectProtocolFeesInstruction;
use crate::instructions::SetPoolCreatorInstruction;
use crate::state::AmmConfig;
use crate::state::Pool;
use crate::token::transfer_signed;

/// Accounts for `CollectProtocolFees`.
#[derive(Accounts, Debug)]
pub struct CollectProtocolFeesAccounts<'a> {
	/// The pool's fee-tier authority.
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	/// The pool's fee tier.
	pub amm_config: &'a AccountView,
	/// The pool.
	pub pool: &'a mut AccountView,
	/// The pool's token 0 vault.
	pub vault_0: &'a mut AccountView,
	/// The pool's token 1 vault.
	pub vault_1: &'a mut AccountView,
	/// Token 0 account that receives the fees.
	pub recipient_token_0: &'a mut AccountView,
	/// Token 1 account that receives the fees.
	pub recipient_token_1: &'a mut AccountView,
	/// Token program that owns token 0.
	pub token_program_0: &'a AccountView,
	/// Token program that owns token 1.
	pub token_program_1: &'a AccountView,
}

/// Accounts for `CollectCreatorFees`.
#[derive(Accounts, Debug)]
pub struct CollectCreatorFeesAccounts<'a> {
	/// The pool's creator.
	#[pina(validate(signer))]
	pub creator: &'a AccountView,
	/// The pool.
	pub pool: &'a mut AccountView,
	/// The pool's token 0 vault.
	pub vault_0: &'a mut AccountView,
	/// The pool's token 1 vault.
	pub vault_1: &'a mut AccountView,
	/// Token 0 account that receives the fees.
	pub recipient_token_0: &'a mut AccountView,
	/// Token 1 account that receives the fees.
	pub recipient_token_1: &'a mut AccountView,
	/// Token program that owns token 0.
	pub token_program_0: &'a AccountView,
	/// Token program that owns token 1.
	pub token_program_1: &'a AccountView,
}

/// Accounts for `SetPoolCreator`.
#[derive(Accounts, Debug)]
pub struct SetPoolCreatorAccounts<'a> {
	/// The pool's current creator.
	#[pina(validate(signer))]
	pub creator: &'a AccountView,
	/// The pool.
	pub pool: &'a mut AccountView,
}

/// The accounts a fee collection pays out of and into.
struct FeeTransfer<'a> {
	vault_0: &'a AccountView,
	vault_1: &'a AccountView,
	recipient_token_0: &'a AccountView,
	recipient_token_1: &'a AccountView,
	token_program_0: &'a AccountView,
	token_program_1: &'a AccountView,
}

impl FeeTransfer<'_> {
	/// Pay `amount_0` and `amount_1` out of the pool's vaults.
	fn pay(
		&self,
		pool: &AccountView,
		snapshot: &PoolSnapshot,
		amount_0: u64,
		amount_1: u64,
	) -> ProgramResult {
		snapshot.assert_vaults(self.vault_0, self.vault_1)?;
		with_pool_signer(snapshot, |signer| {
			if amount_0 > 0 {
				transfer_signed(
					self.vault_0,
					self.recipient_token_0,
					pool,
					self.token_program_0,
					amount_0,
					signer,
				)?;
			}
			if amount_1 > 0 {
				transfer_signed(
					self.vault_1,
					self.recipient_token_1,
					pool,
					self.token_program_1,
					amount_1,
					signer,
				)?;
			}
			Ok(())
		})
	}
}

/// Emit a [`FeesCollected`] event.
fn emit_collection(
	pool: &Address,
	collector: &Address,
	is_protocol: bool,
	amount_0: u64,
	amount_1: u64,
) -> ProgramResult {
	FeesCollected::emit(|event| {
		event.pool = *pool;
		event.collector = *collector;
		event.is_protocol = u8::from(is_protocol);
		event.amount_0.set(amount_0);
		event.amount_1.set(amount_1);
		Ok(())
	})
}

impl<'a> ProcessAccountInfos<'a> for CollectProtocolFeesAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CollectProtocolFeesInstruction::try_from_bytes(data)?;
		let snapshot = PoolSnapshot::load(self.pool)?;
		if self.amm_config.address() != &snapshot.amm_config {
			return Err(AmmError::PoolAccountMismatch.into());
		}
		if &self.amm_config.as_account::<AmmConfig>(&ID)?.authority != self.authority.address() {
			return Err(AmmError::Unauthorized.into());
		}

		let amount_0 = snapshot.protocol_fees_0.min(args.maximum_amount_0.get());
		let amount_1 = snapshot.protocol_fees_1.min(args.maximum_amount_1.get());
		{
			let mut pool = self.pool.as_account_mut::<Pool>(&ID)?;
			pool.protocol_fees_0.set(
				snapshot
					.protocol_fees_0
					.checked_sub(amount_0)
					.ok_or(AmmError::MathOverflow)?,
			);
			pool.protocol_fees_1.set(
				snapshot
					.protocol_fees_1
					.checked_sub(amount_1)
					.ok_or(AmmError::MathOverflow)?,
			);
		}
		FeeTransfer {
			vault_0: self.vault_0,
			vault_1: self.vault_1,
			recipient_token_0: self.recipient_token_0,
			recipient_token_1: self.recipient_token_1,
			token_program_0: self.token_program_0,
			token_program_1: self.token_program_1,
		}
		.pay(self.pool, &snapshot, amount_0, amount_1)?;
		emit_collection(
			self.pool.address(),
			self.authority.address(),
			true,
			amount_0,
			amount_1,
		)
	}
}

impl<'a> ProcessAccountInfos<'a> for CollectCreatorFeesAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CollectCreatorFeesInstruction::try_from_bytes(data)?;
		let snapshot = PoolSnapshot::load(self.pool)?;
		if &snapshot.creator != self.creator.address() {
			return Err(AmmError::Unauthorized.into());
		}

		let amount_0 = snapshot.creator_fees_0.min(args.maximum_amount_0.get());
		let amount_1 = snapshot.creator_fees_1.min(args.maximum_amount_1.get());
		{
			let mut pool = self.pool.as_account_mut::<Pool>(&ID)?;
			pool.creator_fees_0.set(
				snapshot
					.creator_fees_0
					.checked_sub(amount_0)
					.ok_or(AmmError::MathOverflow)?,
			);
			pool.creator_fees_1.set(
				snapshot
					.creator_fees_1
					.checked_sub(amount_1)
					.ok_or(AmmError::MathOverflow)?,
			);
		}
		FeeTransfer {
			vault_0: self.vault_0,
			vault_1: self.vault_1,
			recipient_token_0: self.recipient_token_0,
			recipient_token_1: self.recipient_token_1,
			token_program_0: self.token_program_0,
			token_program_1: self.token_program_1,
		}
		.pay(self.pool, &snapshot, amount_0, amount_1)?;
		emit_collection(
			self.pool.address(),
			self.creator.address(),
			false,
			amount_0,
			amount_1,
		)
	}
}

impl<'a> ProcessAccountInfos<'a> for SetPoolCreatorAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SetPoolCreatorInstruction::try_from_bytes(data)?;
		let mut pool = self.pool.as_account_mut::<Pool>(&ID)?;
		if &pool.creator != self.creator.address() {
			return Err(AmmError::Unauthorized.into());
		}
		pool.creator = args.new_creator;
		Ok(())
	}
}
