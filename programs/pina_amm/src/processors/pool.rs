//! `CreatePool`.

use pina::*;

use crate::ID;
use crate::errors::AmmError;
use crate::events::PoolCreated;
use crate::instructions::CreatePoolInstruction;
use crate::math::initial_liquidity;
use crate::state::AmmConfig;
use crate::state::LP_MINT_DECIMALS;
use crate::state::Pool;
use crate::state::PoolLpMint;
use crate::state::PoolVault;
use crate::token::MINT_LEN;
use crate::token::TOKEN_ACCOUNT_LEN;
use crate::token::assert_supported_mint;
use crate::token::create_pda_account;
use crate::token::transfer;

/// Accounts for `CreatePool`.
#[derive(Accounts, Debug)]
pub struct CreatePoolAccounts<'a> {
	/// Pays rent for the pool, its vaults, its LP mint, and the LP token
	/// account.
	#[pina(validate(signer, writable))]
	pub payer: &'a AccountView,
	/// Owner of the token accounts the initial deposit comes from. May be the
	/// same account as `payer`.
	#[pina(validate(signer))]
	pub depositor: &'a AccountView,
	/// Signer that authorizes pool creation. When the tier restricts pool
	/// creation this must be its pool-creator authority; otherwise any signer
	/// works, so pass the payer.
	#[pina(validate(signer))]
	pub pool_creator_authority: &'a AccountView,
	/// The fee tier the pool is created under.
	pub amm_config: &'a AccountView,
	/// Token 0 mint: the byte-wise smaller mint address.
	pub mint_0: &'a AccountView,
	/// Token 1 mint: the byte-wise larger mint address.
	pub mint_1: &'a AccountView,
	/// The pool PDA to create: `[b"pool", amm_config, mint_0, mint_1]`.
	pub pool: &'a mut AccountView,
	/// The LP mint PDA to create: `[b"pool_lp_mint", pool]`.
	pub lp_mint: &'a mut AccountView,
	/// The token 0 vault PDA to create: `[b"pool_vault", pool, mint_0]`.
	pub vault_0: &'a mut AccountView,
	/// The token 1 vault PDA to create: `[b"pool_vault", pool, mint_1]`.
	pub vault_1: &'a mut AccountView,
	/// The depositor's token 0 account.
	pub depositor_token_0: &'a mut AccountView,
	/// The depositor's token 1 account.
	pub depositor_token_1: &'a mut AccountView,
	/// Wallet that receives the initial LP.
	pub lp_owner: &'a AccountView,
	/// `lp_owner`'s associated token account for the LP mint, created here.
	pub lp_owner_token: &'a mut AccountView,
	/// Token program that owns `mint_0`.
	pub token_program_0: &'a AccountView,
	/// Token program that owns `mint_1`.
	pub token_program_1: &'a AccountView,
	/// SPL Token, which owns every LP mint.
	#[pina(validate(program = token::ID))]
	pub lp_token_program: &'a AccountView,
	/// The associated token account program.
	#[pina(validate(program = associated_token_account::ID))]
	pub associated_token_program: &'a AccountView,
	/// The system program.
	#[pina(validate(program = system::ID))]
	pub system_program: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for CreatePoolAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CreatePoolInstruction::try_from_bytes(data)?;
		let amount_0 = args.amount_0.get();
		let amount_1 = args.amount_1.get();
		let creator = args.creator;
		let creator_fee_mode = args.creator_fee_mode;
		// The default address can never sign, so creator fees accrued to it
		// would be unclaimable forever while still reducing the reserves every
		// withdrawal pays from.
		if creator == Address::default() {
			return Err(AmmError::DefaultCreator.into());
		}

		let config_address = *self.amm_config.address();
		let (pool_creator_authority, trade_fee_rate, protocol_fee_rate, creator_fee_rate) = {
			let config = self.amm_config.as_account::<AmmConfig>(&ID)?;
			(
				config.pool_creator_authority,
				config.trade_fee_rate.get(),
				config.protocol_fee_rate.get(),
				config.creator_fee_rate.get(),
			)
		};
		// `pool_creator_authority` always signs; a restricted tier also pins
		// which signer it must be.
		if pool_creator_authority != Address::default()
			&& self.pool_creator_authority.address() != &pool_creator_authority
		{
			return Err(AmmError::PoolCreatorNotAuthorized.into());
		}

		let mint_0 = *self.mint_0.address();
		let mint_1 = *self.mint_1.address();
		if mint_0.as_ref() >= mint_1.as_ref() {
			return Err(AmmError::InvalidMintOrder.into());
		}
		assert_supported_mint(self.mint_0, self.token_program_0)?;
		assert_supported_mint(self.mint_1, self.token_program_1)?;

		// Derive every address the pool stores before creating anything, so a
		// wrong account fails before rent moves.
		let pool_address = *self.pool.address();
		let (vault_0_address, vault_0_bump) = PoolVault::try_find_pda(&pool_address, &mint_0, &ID)
			.ok_or(AmmError::PoolAccountMismatch)?;
		let (vault_1_address, vault_1_bump) = PoolVault::try_find_pda(&pool_address, &mint_1, &ID)
			.ok_or(AmmError::PoolAccountMismatch)?;
		let (lp_mint_address, lp_mint_bump) =
			PoolLpMint::try_find_pda(&pool_address, &ID).ok_or(AmmError::PoolAccountMismatch)?;
		if self.vault_0.address() != &vault_0_address
			|| self.vault_1.address() != &vault_1_address
			|| self.lp_mint.address() != &lp_mint_address
		{
			return Err(AmmError::PoolAccountMismatch.into());
		}

		let liquidity = initial_liquidity(amount_0, amount_1)?;
		let (_, pool_bump) = CreateProgramAccount {
			account: self.pool,
			payer: self.payer,
			owner: &ID,
			seeds: &Pool::seeds(&config_address, &mint_0, &mint_1).as_slices(),
		}
		.invoke_with_bump::<Pool>(|pool, bump| {
			pool.amm_config = config_address;
			pool.creator = creator;
			pool.mint_0 = mint_0;
			pool.mint_1 = mint_1;
			pool.vault_0 = vault_0_address;
			pool.vault_1 = vault_1_address;
			pool.lp_mint = lp_mint_address;
			pool.lp_supply.set(liquidity.lp_supply);
			pool.trade_fee_rate.set(trade_fee_rate);
			pool.protocol_fee_rate.set(protocol_fee_rate);
			pool.creator_fee_rate.set(creator_fee_rate);
			pool.creator_fee_mode = creator_fee_mode;
			pool.bump = bump;
			Ok(())
		})?;

		create_vault(
			self.payer,
			self.vault_0,
			self.mint_0,
			&pool_address,
			self.token_program_0,
			vault_0_bump,
		)?;
		create_vault(
			self.payer,
			self.vault_1,
			self.mint_1,
			&pool_address,
			self.token_program_1,
			vault_1_bump,
		)?;

		let lp_mint_seeds = PoolLpMint::seeds(&pool_address).with_bump(lp_mint_bump);
		create_pda_account(
			self.payer,
			self.lp_mint,
			MINT_LEN,
			&token::ID,
			&lp_mint_seeds.to_signer().as_signer(),
		)?;
		token::instructions::InitializeMint2::new(
			self.lp_mint,
			LP_MINT_DECIMALS,
			&pool_address,
			None,
		)
		.invoke()?;

		transfer(
			self.depositor_token_0,
			self.vault_0,
			self.depositor,
			self.token_program_0,
			amount_0,
		)?;
		transfer(
			self.depositor_token_1,
			self.vault_1,
			self.depositor,
			self.token_program_1,
			amount_1,
		)?;

		associated_token_account::instructions::CreateIdempotent {
			funding_account: self.payer,
			account: self.lp_owner_token,
			wallet: self.lp_owner,
			mint: self.lp_mint,
			system_program: self.system_program,
			token_program: self.lp_token_program,
		}
		.invoke()?;
		let pool_seeds = Pool::seeds(&config_address, &mint_0, &mint_1).with_bump(pool_bump);
		token::instructions::MintTo::new(
			self.lp_mint,
			self.lp_owner_token,
			self.pool,
			liquidity.lp_minted,
		)
		.invoke_signed(&[pool_seeds.to_signer().as_signer()])?;

		PoolCreated::emit(|event| {
			event.pool = pool_address;
			event.amm_config = config_address;
			event.creator = creator;
			event.mint_0 = mint_0;
			event.mint_1 = mint_1;
			event.lp_mint = lp_mint_address;
			event.amount_0.set(amount_0);
			event.amount_1.set(amount_1);
			event.lp_supply.set(liquidity.lp_supply);
			Ok(())
		})
	}
}

/// Create a pool vault: a token account at `[b"pool_vault", pool, mint]`
/// whose owner is the pool.
fn create_vault(
	payer: &AccountView,
	vault: &AccountView,
	mint: &AccountView,
	pool: &Address,
	token_program: &AccountView,
	bump: u8,
) -> ProgramResult {
	let seeds = PoolVault::seeds(pool, mint.address()).with_bump(bump);
	create_pda_account(
		payer,
		vault,
		TOKEN_ACCOUNT_LEN,
		token_program.address(),
		&seeds.to_signer().as_signer(),
	)?;
	token::instructions::InitializeAccount3::new(vault, mint, pool)
		.invoke_with_program(token_program.address())
}
