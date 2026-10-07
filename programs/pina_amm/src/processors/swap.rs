//! `SwapExactIn` and `SwapExactOut`.

use pina::*;

use super::common::PoolSnapshot;
use super::common::with_pool_signer;
use crate::ID;
use crate::errors::AmmError;
use crate::events::Swapped;
use crate::instructions::SwapExactInInstruction;
use crate::instructions::SwapExactOutInstruction;
use crate::math::SwapQuote;
use crate::math::SwapReserves;
use crate::math::assert_constant_product;
use crate::math::swap_exact_in;
use crate::math::swap_exact_out;
use crate::state::Pool;
use crate::token::transfer;
use crate::token::transfer_signed;

/// Accounts for `SwapExactIn` and `SwapExactOut`.
///
/// The direction is implied by the vaults: pass the pool's token 0 vault as
/// `input_vault` to sell token 0, or its token 1 vault to sell token 1.
#[derive(Accounts, Debug)]
pub struct SwapAccounts<'a> {
	/// The trader, who owns `input_token`.
	#[pina(validate(signer))]
	pub trader: &'a AccountView,
	/// The pool to trade against.
	pub pool: &'a mut AccountView,
	/// The trader's account for the token being sold.
	pub input_token: &'a mut AccountView,
	/// The account that receives the token being bought.
	pub output_token: &'a mut AccountView,
	/// The pool's vault for the token being sold.
	pub input_vault: &'a mut AccountView,
	/// The pool's vault for the token being bought.
	pub output_vault: &'a mut AccountView,
	/// Token program that owns the sold token.
	pub input_token_program: &'a AccountView,
	/// Token program that owns the bought token.
	pub output_token_program: &'a AccountView,
}

/// How a trader's limit applies to a quote.
#[derive(Clone, Copy)]
enum SwapRequest {
	/// Sell `amount_in`; receive at least `minimum_out`.
	ExactIn { amount_in: u64, minimum_out: u64 },
	/// Buy `amount_out`; pay at most `maximum_in`.
	ExactOut { amount_out: u64, maximum_in: u64 },
}

impl SwapAccounts<'_> {
	fn execute(mut self, request: SwapRequest) -> ProgramResult {
		let snapshot = PoolSnapshot::load(self.pool)?;
		let zero_for_one = if self.input_vault.address() == &snapshot.vault_0
			&& self.output_vault.address() == &snapshot.vault_1
		{
			true
		} else if self.input_vault.address() == &snapshot.vault_1
			&& self.output_vault.address() == &snapshot.vault_0
		{
			false
		} else {
			return Err(AmmError::PoolAccountMismatch.into());
		};

		let (vault_0, vault_1, token_program_0, token_program_1) = if zero_for_one {
			(
				&*self.input_vault,
				&*self.output_vault,
				self.input_token_program,
				self.output_token_program,
			)
		} else {
			(
				&*self.output_vault,
				&*self.input_vault,
				self.output_token_program,
				self.input_token_program,
			)
		};
		let (reserve_0, reserve_1) =
			snapshot.reserves(vault_0, vault_1, token_program_0, token_program_1)?;
		let reserves = if zero_for_one {
			SwapReserves {
				input: reserve_0,
				output: reserve_1,
			}
		} else {
			SwapReserves {
				input: reserve_1,
				output: reserve_0,
			}
		};

		let creator_fee_on_input = snapshot.creator_fee_mode.is_on_input(zero_for_one);
		let quote = match request {
			SwapRequest::ExactIn {
				amount_in,
				minimum_out,
			} => {
				let quote =
					swap_exact_in(amount_in, reserves, snapshot.rates, creator_fee_on_input)?;
				if quote.amount_out < minimum_out {
					return Err(AmmError::SlippageExceeded.into());
				}
				quote
			}
			SwapRequest::ExactOut {
				amount_out,
				maximum_in,
			} => {
				let quote =
					swap_exact_out(amount_out, reserves, snapshot.rates, creator_fee_on_input)?;
				if quote.amount_in > maximum_in {
					return Err(AmmError::SlippageExceeded.into());
				}
				quote
			}
		};
		let after = quote.reserves_after(reserves)?;
		assert_constant_product(reserves, after)?;

		self.accrue_fees(&quote, zero_for_one)?;

		transfer(
			self.input_token,
			self.input_vault,
			self.trader,
			self.input_token_program,
			quote.amount_in,
		)?;
		with_pool_signer(&snapshot, |signer| {
			transfer_signed(
				self.output_vault,
				self.output_token,
				self.pool,
				self.output_token_program,
				quote.amount_out,
				signer,
			)
		})?;

		let (reserve_0, reserve_1) = if zero_for_one {
			(after.input, after.output)
		} else {
			(after.output, after.input)
		};
		Swapped::emit(|event| {
			event.pool = *self.pool.address();
			event.trader = *self.trader.address();
			event.zero_for_one = u8::from(zero_for_one);
			event.amount_in.set(quote.amount_in);
			event.amount_out.set(quote.amount_out);
			event.trade_fee.set(quote.trade_fee);
			event.protocol_fee.set(quote.protocol_fee);
			event.creator_fee.set(quote.creator_fee);
			event.creator_fee_on_input = u8::from(creator_fee_on_input);
			event.reserve_0.set(reserve_0);
			event.reserve_1.set(reserve_1);
			Ok(())
		})
	}

	/// Record the protocol and creator fees this swap leaves in the vaults.
	fn accrue_fees(&mut self, quote: &SwapQuote, zero_for_one: bool) -> ProgramResult {
		let mut pool = self.pool.as_account_mut::<Pool>(&ID)?;
		let (protocol_input, creator_input, creator_output) = if quote.creator_fee_on_input {
			(quote.protocol_fee, quote.creator_fee, 0)
		} else {
			(quote.protocol_fee, 0, quote.creator_fee)
		};
		let add = |field: &mut PodU64, amount: u64| -> ProgramResult {
			field.set(
				field
					.get()
					.checked_add(amount)
					.ok_or(AmmError::MathOverflow)?,
			);
			Ok(())
		};
		if zero_for_one {
			add(&mut pool.protocol_fees_0, protocol_input)?;
			add(&mut pool.creator_fees_0, creator_input)?;
			add(&mut pool.creator_fees_1, creator_output)?;
		} else {
			add(&mut pool.protocol_fees_1, protocol_input)?;
			add(&mut pool.creator_fees_1, creator_input)?;
			add(&mut pool.creator_fees_0, creator_output)?;
		}
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for SwapAccounts<'a> {
	/// Routes to the exact-input or exact-output handler from the
	/// instruction's discriminator.
	fn process(self, data: &[u8]) -> ProgramResult {
		match data.first().copied() {
			Some(discriminator)
				if discriminator == crate::instructions::AmmInstruction::SwapExactIn as u8 =>
			{
				let args = SwapExactInInstruction::try_from_bytes(data)?;
				self.execute(SwapRequest::ExactIn {
					amount_in: args.amount_in.get(),
					minimum_out: args.minimum_amount_out.get(),
				})
			}
			_ => {
				let args = SwapExactOutInstruction::try_from_bytes(data)?;
				self.execute(SwapRequest::ExactOut {
					amount_out: args.amount_out.get(),
					maximum_in: args.maximum_amount_in.get(),
				})
			}
		}
	}
}
