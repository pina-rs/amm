//! `CreateConfig` and `UpdateConfig`.

use pina::*;

use crate::ID;
use crate::errors::AmmError;
use crate::instructions::CreateConfigInstruction;
use crate::instructions::UpdateConfigInstruction;
use crate::math::FeeRates;
use crate::state::AmmConfig;
use crate::upgrade_authority::assert_upgrade_authority;

/// Accounts for `CreateConfig`.
#[derive(Accounts, Debug)]
pub struct CreateConfigAccounts<'a> {
	/// Pays the new tier's rent.
	#[pina(validate(signer, writable))]
	pub payer: &'a AccountView,
	/// The program's upgrade authority.
	#[pina(validate(signer))]
	pub upgrade_authority: &'a AccountView,
	/// The program's program-data account, which records its upgrade
	/// authority.
	pub program_data: &'a AccountView,
	/// The tier PDA to create: `[b"amm_config", index]`.
	pub amm_config: &'a mut AccountView,
	/// The system program.
	#[pina(validate(program = system::ID))]
	pub system_program: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for CreateConfigAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CreateConfigInstruction::try_from_bytes(data)?;
		assert_upgrade_authority(self.program_data, self.upgrade_authority)?;
		let rates = FeeRates {
			trade: args.trade_fee_rate.get(),
			protocol: args.protocol_fee_rate.get(),
			creator: args.creator_fee_rate.get(),
		}
		.validate()?;
		let index = args.index.get();
		let authority = args.authority;
		let pool_creator_authority = args.pool_creator_authority;
		if authority == Address::default() {
			return Err(AmmError::Unauthorized.into());
		}

		CreateProgramAccount {
			account: self.amm_config,
			payer: self.payer,
			owner: &ID,
			seeds: &AmmConfig::seeds(index).as_slices(),
		}
		.invoke_with_bump::<AmmConfig>(|config, bump| {
			config.authority = authority;
			config.pool_creator_authority = pool_creator_authority;
			config.trade_fee_rate.set(rates.trade);
			config.protocol_fee_rate.set(rates.protocol);
			config.creator_fee_rate.set(rates.creator);
			config.index.set(index);
			config.bump = bump;
			Ok(())
		})?;
		Ok(())
	}
}

/// Accounts for `UpdateConfig`.
#[derive(Accounts, Debug)]
pub struct UpdateConfigAccounts<'a> {
	/// The tier's current authority.
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	/// The tier to update.
	pub amm_config: &'a mut AccountView,
}

impl<'a> ProcessAccountInfos<'a> for UpdateConfigAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = UpdateConfigInstruction::try_from_bytes(data)?;
		let rates = FeeRates {
			trade: args.trade_fee_rate.get(),
			protocol: args.protocol_fee_rate.get(),
			creator: args.creator_fee_rate.get(),
		}
		.validate()?;
		if args.new_authority == Address::default() {
			return Err(AmmError::Unauthorized.into());
		}

		let mut config = self.amm_config.as_account_mut::<AmmConfig>(&ID)?;
		if &config.authority != self.authority.address() {
			return Err(AmmError::Unauthorized.into());
		}
		config.authority = args.new_authority;
		config.trade_fee_rate.set(rates.trade);
		config.protocol_fee_rate.set(rates.protocol);
		config.creator_fee_rate.set(rates.creator);
		Ok(())
	}
}
