//! `Deposit` and `Withdraw`.

use pina::*;

use super::common::PoolSnapshot;
use super::common::with_pool_signer;
use crate::ID;
use crate::errors::AmmError;
use crate::events::LiquidityChanged;
use crate::instructions::AmmInstruction;
use crate::instructions::DepositInstruction;
use crate::instructions::WithdrawInstruction;
use crate::math::TokenAmounts;
use crate::math::deposit_amounts;
use crate::math::withdraw_amounts;
use crate::state::Pool;
use crate::token::transfer;
use crate::token::transfer_signed;

/// Accounts for `Deposit` and `Withdraw`.
#[derive(Accounts, Debug)]
pub struct LiquidityAccounts<'a> {
	/// The LP owner, who owns every token account below.
	#[pina(validate(signer))]
	pub owner: &'a AccountView,
	/// The pool.
	pub pool: &'a mut AccountView,
	/// The pool's token 0 vault.
	pub vault_0: &'a mut AccountView,
	/// The pool's token 1 vault.
	pub vault_1: &'a mut AccountView,
	/// The pool's LP mint.
	pub lp_mint: &'a mut AccountView,
	/// The owner's token 0 account.
	pub owner_token_0: &'a mut AccountView,
	/// The owner's token 1 account.
	pub owner_token_1: &'a mut AccountView,
	/// The owner's LP token account.
	pub owner_lp_token: &'a mut AccountView,
	/// Token program that owns token 0.
	pub token_program_0: &'a AccountView,
	/// Token program that owns token 1.
	pub token_program_1: &'a AccountView,
	/// SPL Token, which owns every LP mint.
	#[pina(validate(program = token::ID))]
	pub lp_token_program: &'a AccountView,
}

impl LiquidityAccounts<'_> {
	fn load(&self) -> Result<(PoolSnapshot, u64, u64), ProgramError> {
		let snapshot = PoolSnapshot::load(self.pool)?;
		snapshot.assert_vaults(self.vault_0, self.vault_1)?;
		if self.lp_mint.address() != &snapshot.lp_mint {
			return Err(AmmError::PoolAccountMismatch.into());
		}
		let (reserve_0, reserve_1) = snapshot.reserves(
			self.vault_0,
			self.vault_1,
			self.token_program_0,
			self.token_program_1,
		)?;
		Ok((snapshot, reserve_0, reserve_1))
	}

	fn deposit(self, lp_amount: u64, maximum_0: u64, maximum_1: u64) -> ProgramResult {
		let (snapshot, reserve_0, reserve_1) = self.load()?;
		let amounts = deposit_amounts(lp_amount, snapshot.lp_supply, reserve_0, reserve_1)?;
		if amounts.amount_0 > maximum_0 || amounts.amount_1 > maximum_1 {
			return Err(AmmError::SlippageExceeded.into());
		}
		let lp_supply = snapshot
			.lp_supply
			.checked_add(lp_amount)
			.ok_or(AmmError::MathOverflow)?;
		self.pool
			.as_account_mut::<Pool>(&ID)?
			.lp_supply
			.set(lp_supply);

		transfer(
			self.owner_token_0,
			self.vault_0,
			self.owner,
			self.token_program_0,
			amounts.amount_0,
		)?;
		transfer(
			self.owner_token_1,
			self.vault_1,
			self.owner,
			self.token_program_1,
			amounts.amount_1,
		)?;
		with_pool_signer(&snapshot, |signer| {
			token::instructions::MintTo::new(
				self.lp_mint,
				self.owner_lp_token,
				self.pool,
				lp_amount,
			)
			.invoke_signed(core::slice::from_ref(signer))
		})?;

		self.emit(true, lp_amount, amounts, lp_supply)
	}

	fn withdraw(self, lp_amount: u64, minimum_0: u64, minimum_1: u64) -> ProgramResult {
		let (snapshot, reserve_0, reserve_1) = self.load()?;
		let amounts = withdraw_amounts(lp_amount, snapshot.lp_supply, reserve_0, reserve_1)?;
		if amounts.amount_0 < minimum_0 || amounts.amount_1 < minimum_1 {
			return Err(AmmError::SlippageExceeded.into());
		}
		let lp_supply = snapshot
			.lp_supply
			.checked_sub(lp_amount)
			.ok_or(AmmError::InsufficientLiquidity)?;
		self.pool
			.as_account_mut::<Pool>(&ID)?
			.lp_supply
			.set(lp_supply);

		token::instructions::Burn::new(self.owner_lp_token, self.lp_mint, self.owner, lp_amount)
			.invoke()?;
		with_pool_signer(&snapshot, |signer| {
			if amounts.amount_0 > 0 {
				transfer_signed(
					self.vault_0,
					self.owner_token_0,
					self.pool,
					self.token_program_0,
					amounts.amount_0,
					signer,
				)?;
			}
			if amounts.amount_1 > 0 {
				transfer_signed(
					self.vault_1,
					self.owner_token_1,
					self.pool,
					self.token_program_1,
					amounts.amount_1,
					signer,
				)?;
			}
			Ok(())
		})?;

		self.emit(false, lp_amount, amounts, lp_supply)
	}

	fn emit(
		&self,
		is_deposit: bool,
		lp_amount: u64,
		amounts: TokenAmounts,
		lp_supply: u64,
	) -> ProgramResult {
		LiquidityChanged::emit(|event| {
			event.pool = *self.pool.address();
			event.owner = *self.owner.address();
			event.is_deposit = u8::from(is_deposit);
			event.lp_amount.set(lp_amount);
			event.amount_0.set(amounts.amount_0);
			event.amount_1.set(amounts.amount_1);
			event.lp_supply.set(lp_supply);
			Ok(())
		})
	}
}

impl<'a> ProcessAccountInfos<'a> for LiquidityAccounts<'a> {
	/// Routes to the deposit or withdrawal handler from the instruction's
	/// discriminator.
	fn process(self, data: &[u8]) -> ProgramResult {
		if data.first().copied() == Some(AmmInstruction::Deposit as u8) {
			let args = DepositInstruction::try_from_bytes(data)?;
			return self.deposit(
				args.lp_amount.get(),
				args.maximum_amount_0.get(),
				args.maximum_amount_1.get(),
			);
		}
		let args = WithdrawInstruction::try_from_bytes(data)?;
		self.withdraw(
			args.lp_amount.get(),
			args.minimum_amount_0.get(),
			args.minimum_amount_1.get(),
		)
	}
}
