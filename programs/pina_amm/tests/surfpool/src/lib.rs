#![cfg(test)]

//! End-to-end tests for the Pina AMM.
//!
//! Every test starts an isolated, offline Surfnet, deploys the compiled SBF
//! program, and drives it only through the generated Rust client, so the
//! suite also proves that the client's account lists and data layouts match
//! the program. Token fixtures use the real SPL Token and Token-2022 programs.
//!
//! Run with `pina test` (or `devenv shell test:surfpool`) after `pina build`.

mod cli;
mod harness;

use harness::Harness;
use harness::MintExtension;
use harness::TOKEN_2022_PROGRAM;
use harness::TOKEN_PROGRAM;
use harness::ata;
use pina_amm_client::PINA_AMM_ID;
use pina_amm_client::accounts::AmmConfig;
use pina_amm_client::accounts::Pool;
use pina_amm_client::instructions::CollectCreatorFees;
use pina_amm_client::instructions::CollectCreatorFeesInstructionData;
use pina_amm_client::instructions::CollectProtocolFees;
use pina_amm_client::instructions::CollectProtocolFeesInstructionData;
use pina_amm_client::instructions::CreateConfig;
use pina_amm_client::instructions::CreateConfigInstructionData;
use pina_amm_client::instructions::CreatePool;
use pina_amm_client::instructions::CreatePoolInstructionData;
use pina_amm_client::instructions::Deposit;
use pina_amm_client::instructions::DepositInstructionData;
use pina_amm_client::instructions::SetPoolCreator;
use pina_amm_client::instructions::SetPoolCreatorInstructionData;
use pina_amm_client::instructions::SwapExactIn;
use pina_amm_client::instructions::SwapExactInInstructionData;
use pina_amm_client::instructions::SwapExactOut;
use pina_amm_client::instructions::SwapExactOutInstructionData;
use pina_amm_client::instructions::UpdateConfig;
use pina_amm_client::instructions::UpdateConfigInstructionData;
use pina_amm_client::instructions::Withdraw;
use pina_amm_client::instructions::WithdrawInstructionData;
use pina_test::Instruction;
use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;

const TRADE_FEE_RATE: u32 = 2_500;
const PROTOCOL_FEE_RATE: u32 = 200_000;
const CREATOR_FEE_RATE: u32 = 500;
const MINIMUM_LIQUIDITY: u64 = 1_000;
const DENOMINATOR: u128 = 1_000_000;

/// Program error codes, mirrored from `PinaAmmError` so assertions read as
/// names.
mod code {
	pub const INVALID_FEE_RATES: u32 = 0;
	pub const UNAUTHORIZED: u32 = 1;
	pub const INVALID_MINT_ORDER: u32 = 3;
	pub const UNSUPPORTED_MINT: u32 = 4;
	pub const POOL_ACCOUNT_MISMATCH: u32 = 5;
	pub const SLIPPAGE_EXCEEDED: u32 = 9;
	pub const INSUFFICIENT_LIQUIDITY: u32 = 10;
	pub const POOL_CREATOR_NOT_AUTHORIZED: u32 = 14;
}

fn config_address(index: u16) -> Pubkey {
	AmmConfig::find_pda(index).0
}

fn vault_address(pool: &Pubkey, mint: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(&[b"pool_vault", pool.as_ref(), mint.as_ref()], &PINA_AMM_ID).0
}

fn lp_mint_address(pool: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(&[b"pool_lp_mint", pool.as_ref()], &PINA_AMM_ID).0
}

fn create_config_instruction(
	payer: &Pubkey,
	upgrade_authority: &Pubkey,
	index: u16,
	rates: (u32, u32, u32),
	authority: &Pubkey,
	pool_creator_authority: &Pubkey,
) -> Instruction {
	let data = CreateConfigInstructionData::new(|data| {
		data.index.set(index);
		data.trade_fee_rate.set(rates.0);
		data.protocol_fee_rate.set(rates.1);
		data.creator_fee_rate.set(rates.2);
		data.authority = *authority;
		data.pool_creator_authority = *pool_creator_authority;
	})
	.expect("create config data");
	CreateConfig::new(
		*payer,
		*upgrade_authority,
		Harness::program_data(),
		config_address(index),
	)
	.instruction(data)
}

/// The AMM admin: the program's upgrade authority, which also owns every tier
/// it creates in these tests.
fn install_admin(h: &Harness) -> Keypair {
	let admin = h.funded_keypair().expect("admin");
	h.set_upgrade_authority(&admin.pubkey())
		.expect("set upgrade authority");
	admin
}

fn create_config(
	h: &Harness,
	admin: &Keypair,
	index: u16,
	pool_creator_authority: &Pubkey,
) -> Pubkey {
	let instruction = create_config_instruction(
		&h.payer().pubkey(),
		&admin.pubkey(),
		index,
		(TRADE_FEE_RATE, PROTOCOL_FEE_RATE, CREATOR_FEE_RATE),
		&admin.pubkey(),
		pool_creator_authority,
	);
	h.send(&[instruction], &[admin]).expect("create config");
	config_address(index)
}

/// Everything a test needs to talk to one pool.
struct PoolFixture {
	config: Pubkey,
	pool: Pubkey,
	mint_0: Pubkey,
	mint_1: Pubkey,
	program_0: Pubkey,
	program_1: Pubkey,
	vault_0: Pubkey,
	vault_1: Pubkey,
	lp_mint: Pubkey,
	/// The first depositor, who is also the pool's creator.
	owner: Keypair,
	owner_token_0: Pubkey,
	owner_token_1: Pubkey,
	owner_lp: Pubkey,
}

/// Two fresh mints ordered as the program requires.
fn ordered_mints(
	h: &Harness,
	program_a: &Pubkey,
	extensions_a: &[MintExtension],
	program_b: &Pubkey,
	extensions_b: &[MintExtension],
) -> ((Pubkey, Pubkey), (Pubkey, Pubkey)) {
	let a = h.create_mint(program_a, 6, extensions_a).expect("mint a");
	let b = h.create_mint(program_b, 9, extensions_b).expect("mint b");
	if a < b {
		((a, *program_a), (b, *program_b))
	} else {
		((b, *program_b), (a, *program_a))
	}
}

struct PoolSpec<'a> {
	config: Pubkey,
	token_0: (Pubkey, Pubkey),
	token_1: (Pubkey, Pubkey),
	amounts: (u64, u64),
	creator_fee_mode: u8,
	pool_creator_authority: Option<&'a Keypair>,
}

fn create_pool_instruction(
	h: &Harness,
	spec: &PoolSpec<'_>,
	owner: &Keypair,
) -> (Instruction, PoolFixture) {
	let (mint_0, program_0) = spec.token_0;
	let (mint_1, program_1) = spec.token_1;
	let owner_token_0 = h
		.mint_to_owner(&mint_0, &owner.pubkey(), &program_0, spec.amounts.0 * 10)
		.expect("fund token 0");
	let owner_token_1 = h
		.mint_to_owner(&mint_1, &owner.pubkey(), &program_1, spec.amounts.1 * 10)
		.expect("fund token 1");
	let pool = Pool::find_pda(&spec.config, &mint_0, &mint_1).0;
	let lp_mint = lp_mint_address(&pool);
	let owner_lp = ata(&owner.pubkey(), &lp_mint, &TOKEN_PROGRAM);
	let fixture = PoolFixture {
		config: spec.config,
		pool,
		mint_0,
		mint_1,
		program_0,
		program_1,
		vault_0: vault_address(&pool, &mint_0),
		vault_1: vault_address(&pool, &mint_1),
		lp_mint,
		owner: owner.insecure_clone(),
		owner_token_0,
		owner_token_1,
		owner_lp,
	};
	let authority = spec
		.pool_creator_authority
		.map_or_else(|| owner.pubkey(), Signer::pubkey);
	let accounts = CreatePool::new(
		owner.pubkey(),
		owner.pubkey(),
		authority,
		spec.config,
		mint_0,
		mint_1,
		lp_mint,
		fixture.vault_0,
		fixture.vault_1,
		owner_token_0,
		owner_token_1,
		owner.pubkey(),
		owner_lp,
		program_0,
		program_1,
	);
	let data = CreatePoolInstructionData::new(|data| {
		data.amount0.set(spec.amounts.0);
		data.amount1.set(spec.amounts.1);
		data.creator = owner.pubkey();
		data.creator_fee_mode = spec.creator_fee_mode;
	})
	.expect("create pool data");
	(accounts.instruction(data), fixture)
}

fn create_pool(h: &Harness, spec: &PoolSpec<'_>) -> PoolFixture {
	let owner = h.funded_keypair().expect("owner");
	let (instruction, fixture) = create_pool_instruction(h, spec, &owner);
	let mut signers: Vec<&dyn Signer> = vec![&owner];
	if let Some(authority) = spec.pool_creator_authority {
		signers.push(authority);
	}
	h.send(&[instruction], &signers).expect("create pool");
	fixture
}

/// A tier and an SPL Token pool with reserves of 1,000,000,000 and
/// 4,000,000,000.
fn spl_pool(h: &Harness, admin: &Keypair, creator_fee_mode: u8) -> PoolFixture {
	let config = create_config(h, admin, 0, &Pubkey::default());
	let (token_0, token_1) = ordered_mints(h, &TOKEN_PROGRAM, &[], &TOKEN_PROGRAM, &[]);
	create_pool(
		h,
		&PoolSpec {
			config,
			token_0,
			token_1,
			amounts: (1_000_000_000, 4_000_000_000),
			creator_fee_mode,
			pool_creator_authority: None,
		},
	)
}

fn pool_state(h: &Harness, pool: &Pubkey) -> Pool {
	let account = h.account(pool).expect("pool account");
	assert_eq!(account.owner, PINA_AMM_ID, "pool must be program owned");
	let state = Pool::from_bytes(&account.data).expect("decode pool");
	Pool {
		discriminator: state.discriminator,
		migration_version: state.migration_version,
		amm_config: state.amm_config,
		creator: state.creator,
		mint0: state.mint0,
		mint1: state.mint1,
		vault0: state.vault0,
		vault1: state.vault1,
		lp_mint: state.lp_mint,
		lp_supply: state.lp_supply.get(),
		protocol_fees0: state.protocol_fees0.get(),
		protocol_fees1: state.protocol_fees1.get(),
		creator_fees0: state.creator_fees0.get(),
		creator_fees1: state.creator_fees1.get(),
		trade_fee_rate: state.trade_fee_rate.get(),
		protocol_fee_rate: state.protocol_fee_rate.get(),
		creator_fee_rate: state.creator_fee_rate.get(),
		creator_fee_mode: state.creator_fee_mode,
		bump: state.bump,
	}
}

/// Liquidity reserves: vault balances minus fees accrued for collection.
fn reserves(h: &Harness, fixture: &PoolFixture) -> (u64, u64) {
	let state = pool_state(h, &fixture.pool);
	(
		h.token_balance(&fixture.vault_0) - state.protocol_fees0 - state.creator_fees0,
		h.token_balance(&fixture.vault_1) - state.protocol_fees1 - state.creator_fees1,
	)
}

fn swap_exact_in_instruction(
	fixture: &PoolFixture,
	trader: &Pubkey,
	zero_for_one: bool,
	amount_in: u64,
	minimum_amount_out: u64,
) -> Instruction {
	let (input_token, output_token, input_vault, output_vault, input_program, output_program) =
		swap_route(fixture, trader, zero_for_one);
	SwapExactIn::new(
		*trader,
		fixture.pool,
		input_token,
		output_token,
		input_vault,
		output_vault,
		input_program,
		output_program,
	)
	.instruction(
		SwapExactInInstructionData::new(|data| {
			data.amount_in.set(amount_in);
			data.minimum_amount_out.set(minimum_amount_out);
		})
		.expect("swap data"),
	)
}

fn swap_exact_out_instruction(
	fixture: &PoolFixture,
	trader: &Pubkey,
	zero_for_one: bool,
	amount_out: u64,
	maximum_amount_in: u64,
) -> Instruction {
	let (input_token, output_token, input_vault, output_vault, input_program, output_program) =
		swap_route(fixture, trader, zero_for_one);
	SwapExactOut::new(
		*trader,
		fixture.pool,
		input_token,
		output_token,
		input_vault,
		output_vault,
		input_program,
		output_program,
	)
	.instruction(
		SwapExactOutInstructionData::new(|data| {
			data.amount_out.set(amount_out);
			data.maximum_amount_in.set(maximum_amount_in);
		})
		.expect("swap data"),
	)
}

#[allow(clippy::type_complexity)]
fn swap_route(
	fixture: &PoolFixture,
	trader: &Pubkey,
	zero_for_one: bool,
) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
	let token_0 = ata(trader, &fixture.mint_0, &fixture.program_0);
	let token_1 = ata(trader, &fixture.mint_1, &fixture.program_1);
	if zero_for_one {
		(
			token_0,
			token_1,
			fixture.vault_0,
			fixture.vault_1,
			fixture.program_0,
			fixture.program_1,
		)
	} else {
		(
			token_1,
			token_0,
			fixture.vault_1,
			fixture.vault_0,
			fixture.program_1,
			fixture.program_0,
		)
	}
}

fn liquidity_accounts(fixture: &PoolFixture, owner: &Pubkey) -> (Pubkey, Pubkey, Pubkey) {
	(
		ata(owner, &fixture.mint_0, &fixture.program_0),
		ata(owner, &fixture.mint_1, &fixture.program_1),
		ata(owner, &fixture.lp_mint, &TOKEN_PROGRAM),
	)
}

fn deposit_instruction(
	fixture: &PoolFixture,
	owner: &Pubkey,
	lp_amount: u64,
	maximum: (u64, u64),
) -> Instruction {
	let (token_0, token_1, lp) = liquidity_accounts(fixture, owner);
	Deposit::new(
		*owner,
		fixture.pool,
		fixture.vault_0,
		fixture.vault_1,
		fixture.lp_mint,
		token_0,
		token_1,
		lp,
		fixture.program_0,
		fixture.program_1,
	)
	.instruction(
		DepositInstructionData::new(|data| {
			data.lp_amount.set(lp_amount);
			data.maximum_amount0.set(maximum.0);
			data.maximum_amount1.set(maximum.1);
		})
		.expect("deposit data"),
	)
}

fn withdraw_instruction(
	fixture: &PoolFixture,
	owner: &Pubkey,
	lp_amount: u64,
	minimum: (u64, u64),
) -> Instruction {
	let (token_0, token_1, lp) = liquidity_accounts(fixture, owner);
	Withdraw::new(
		*owner,
		fixture.pool,
		fixture.vault_0,
		fixture.vault_1,
		fixture.lp_mint,
		token_0,
		token_1,
		lp,
		fixture.program_0,
		fixture.program_1,
	)
	.instruction(
		WithdrawInstructionData::new(|data| {
			data.lp_amount.set(lp_amount);
			data.minimum_amount0.set(minimum.0);
			data.minimum_amount1.set(minimum.1);
		})
		.expect("withdraw data"),
	)
}

/// A trader holding `amount` of both pool tokens.
fn funded_trader(h: &Harness, fixture: &PoolFixture, amount: u64) -> Keypair {
	let trader = h.funded_keypair().expect("trader");
	h.mint_to_owner(
		&fixture.mint_0,
		&trader.pubkey(),
		&fixture.program_0,
		amount,
	)
	.expect("fund trader token 0");
	h.mint_to_owner(
		&fixture.mint_1,
		&trader.pubkey(),
		&fixture.program_1,
		amount,
	)
	.expect("fund trader token 1");
	trader
}

/// An independent re-derivation of the exact-input formula from the docs,
/// used as the oracle for on-chain results.
fn expected_exact_in(
	amount_in: u64,
	reserve_in: u64,
	reserve_out: u64,
	creator_on_input: bool,
) -> (u64, u64, u64, u64) {
	let amount = u128::from(amount_in);
	let (trade_fee, input_creator_fee) = if creator_on_input {
		let total = (amount * u128::from(TRADE_FEE_RATE + CREATOR_FEE_RATE)).div_ceil(DENOMINATOR);
		let creator =
			total * u128::from(CREATOR_FEE_RATE) / u128::from(TRADE_FEE_RATE + CREATOR_FEE_RATE);
		(total - creator, creator)
	} else {
		(
			(amount * u128::from(TRADE_FEE_RATE)).div_ceil(DENOMINATOR),
			0,
		)
	};
	let net = amount - trade_fee - input_creator_fee;
	let gross_out = net * u128::from(reserve_out) / (u128::from(reserve_in) + net);
	let output_creator_fee = if creator_on_input {
		0
	} else {
		(gross_out * u128::from(CREATOR_FEE_RATE)).div_ceil(DENOMINATOR)
	};
	let protocol = trade_fee * u128::from(PROTOCOL_FEE_RATE) / DENOMINATOR;
	(
		u64::try_from(gross_out - output_creator_fee).expect("out"),
		u64::try_from(trade_fee).expect("trade"),
		u64::try_from(protocol).expect("protocol"),
		u64::try_from(input_creator_fee + output_creator_fee).expect("creator"),
	)
}

#[test]
#[ignore = "run with `pina test`"]
fn create_config_requires_the_upgrade_authority() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = h.funded_keypair().expect("admin");
		let instruction = create_config_instruction(
			&h.payer().pubkey(),
			&admin.pubkey(),
			0,
			(TRADE_FEE_RATE, PROTOCOL_FEE_RATE, CREATOR_FEE_RATE),
			&admin.pubkey(),
			&Pubkey::default(),
		);
		h.expect_custom_error(
			std::slice::from_ref(&instruction),
			&[&admin],
			code::UNAUTHORIZED,
		);

		h.set_upgrade_authority(&admin.pubkey())
			.expect("set authority");
		let too_expensive = create_config_instruction(
			&h.payer().pubkey(),
			&admin.pubkey(),
			0,
			(90_000, PROTOCOL_FEE_RATE, 10_001),
			&admin.pubkey(),
			&Pubkey::default(),
		);
		h.expect_custom_error(&[too_expensive], &[&admin], code::INVALID_FEE_RATES);

		h.send(&[instruction], &[&admin]).expect("create config");
		let account = h.account(&config_address(0)).expect("config account");
		let config = AmmConfig::from_bytes(&account.data).expect("decode config");
		assert_eq!(config.authority, admin.pubkey());
		assert_eq!(config.pool_creator_authority, Pubkey::default());
		assert_eq!(config.trade_fee_rate.get(), TRADE_FEE_RATE);
		assert_eq!(config.protocol_fee_rate.get(), PROTOCOL_FEE_RATE);
		assert_eq!(config.creator_fee_rate.get(), CREATOR_FEE_RATE);
		assert_eq!(config.index.get(), 0);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn tier_updates_only_affect_future_pools() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let stranger = h.funded_keypair().expect("stranger");
		let update = |authority: &Pubkey, new_authority: &Pubkey| {
			UpdateConfig::new(*authority, fixture.config).instruction(
				UpdateConfigInstructionData::new(|data| {
					data.new_authority = *new_authority;
					data.trade_fee_rate.set(10_000);
					data.protocol_fee_rate.set(0);
					data.creator_fee_rate.set(0);
				})
				.expect("update data"),
			)
		};
		h.expect_custom_error(
			&[update(&stranger.pubkey(), &stranger.pubkey())],
			&[&stranger],
			code::UNAUTHORIZED,
		);
		h.send(&[update(&admin.pubkey(), &stranger.pubkey())], &[&admin])
			.expect("update config");

		let account = h.account(&fixture.config).expect("config");
		let config = AmmConfig::from_bytes(&account.data).expect("decode config");
		assert_eq!(config.authority, stranger.pubkey());
		assert_eq!(config.trade_fee_rate.get(), 10_000);
		let pool = pool_state(&h, &fixture.pool);
		assert_eq!(
			pool.trade_fee_rate, TRADE_FEE_RATE,
			"existing pools keep their rates"
		);
		assert_eq!(pool.creator_fee_rate, CREATOR_FEE_RATE);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn create_pool_locks_minimum_liquidity_and_snapshots_the_tier() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 2);
		let pool = pool_state(&h, &fixture.pool);
		// sqrt(1e9 * 4e9) = 2e9
		assert_eq!(pool.lp_supply, 2_000_000_000);
		assert_eq!(
			h.token_balance(&fixture.owner_lp),
			2_000_000_000 - MINIMUM_LIQUIDITY
		);
		assert_eq!(
			h.mint_supply(&fixture.lp_mint),
			2_000_000_000 - MINIMUM_LIQUIDITY
		);
		assert_eq!(h.token_balance(&fixture.vault_0), 1_000_000_000);
		assert_eq!(h.token_balance(&fixture.vault_1), 4_000_000_000);
		assert_eq!(pool.amm_config, fixture.config);
		assert_eq!(pool.creator, fixture.owner.pubkey());
		assert_eq!(pool.mint0, fixture.mint_0);
		assert_eq!(pool.mint1, fixture.mint_1);
		assert_eq!(pool.vault0, fixture.vault_0);
		assert_eq!(pool.vault1, fixture.vault_1);
		assert_eq!(pool.lp_mint, fixture.lp_mint);
		assert_eq!(pool.creator_fee_mode, 2);
		assert_eq!(pool.trade_fee_rate, TRADE_FEE_RATE);
		assert_eq!(pool.protocol_fee_rate, PROTOCOL_FEE_RATE);
		assert_eq!(pool.creator_fee_rate, CREATOR_FEE_RATE);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn create_pool_rejects_unordered_mints_and_unsupported_extensions() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let config = create_config(&h, &admin, 0, &Pubkey::default());
		let owner = h.funded_keypair().expect("owner");

		let (token_0, token_1) = ordered_mints(&h, &TOKEN_PROGRAM, &[], &TOKEN_PROGRAM, &[]);
		let (reversed, _) = create_pool_instruction(
			&h,
			&PoolSpec {
				config,
				token_0: token_1,
				token_1: token_0,
				amounts: (1_000_000, 1_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: None,
			},
			&owner,
		);
		h.expect_custom_error(&[reversed], &[&owner], code::INVALID_MINT_ORDER);

		let (token_0, token_1) = ordered_mints(
			&h,
			&TOKEN_2022_PROGRAM,
			&[MintExtension::TransferFee],
			&TOKEN_PROGRAM,
			&[],
		);
		let (fee_mint, _) = create_pool_instruction(
			&h,
			&PoolSpec {
				config,
				token_0,
				token_1,
				amounts: (1_000_000, 1_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: None,
			},
			&owner,
		);
		h.expect_custom_error(&[fee_mint], &[&owner], code::UNSUPPORTED_MINT);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn token_2022_metadata_mints_trade_like_spl_tokens() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let config = create_config(&h, &admin, 0, &Pubkey::default());
		let (token_0, token_1) = ordered_mints(
			&h,
			&TOKEN_2022_PROGRAM,
			&[MintExtension::MetadataPointer],
			&TOKEN_PROGRAM,
			&[],
		);
		let fixture = create_pool(
			&h,
			&PoolSpec {
				config,
				token_0,
				token_1,
				amounts: (5_000_000_000, 5_000_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: None,
			},
		);
		let trader = funded_trader(&h, &fixture, 1_000_000_000);
		for zero_for_one in [true, false] {
			let before = reserves(&h, &fixture);
			let (reserve_in, reserve_out) = if zero_for_one {
				before
			} else {
				(before.1, before.0)
			};
			let (expected_out, ..) = expected_exact_in(10_000_000, reserve_in, reserve_out, true);
			let output = swap_route(&fixture, &trader.pubkey(), zero_for_one).1;
			let output_before = h.token_balance(&output);
			h.send(
				&[swap_exact_in_instruction(
					&fixture,
					&trader.pubkey(),
					zero_for_one,
					10_000_000,
					expected_out,
				)],
				&[&trader],
			)
			.expect("swap");
			assert_eq!(h.token_balance(&output) - output_before, expected_out);
		}
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn swaps_match_the_documented_formulas_and_accrue_fees() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		// Mode 1 pays the creator in token 0 in both directions.
		let fixture = spl_pool(&h, &admin, 1);
		let trader = funded_trader(&h, &fixture, 1_000_000_000);

		// Sell token 0: the creator fee comes from the input.
		let (reserve_0, reserve_1) = reserves(&h, &fixture);
		let (out, trade, protocol, creator) =
			expected_exact_in(100_000_000, reserve_0, reserve_1, true);
		let token_1 = ata(&trader.pubkey(), &fixture.mint_1, &fixture.program_1);
		let before = h.token_balance(&token_1);
		h.send(
			&[swap_exact_in_instruction(
				&fixture,
				&trader.pubkey(),
				true,
				100_000_000,
				out,
			)],
			&[&trader],
		)
		.expect("sell token 0");
		assert_eq!(h.token_balance(&token_1) - before, out);
		let pool = pool_state(&h, &fixture.pool);
		assert_eq!(pool.protocol_fees0, protocol);
		assert_eq!(pool.creator_fees0, creator);
		assert!(trade > protocol);
		let (after_0, after_1) = reserves(&h, &fixture);
		assert_eq!(after_0, reserve_0 + 100_000_000 - protocol - creator);
		assert_eq!(after_1, reserve_1 - out);
		assert!(
			u128::from(after_0) * u128::from(after_1)
				>= u128::from(reserve_0) * u128::from(reserve_1)
		);

		// Sell token 1: the creator fee now comes from the token 0 output.
		let (reserve_0, reserve_1) = reserves(&h, &fixture);
		let (out, _, protocol_1, creator_0) =
			expected_exact_in(200_000_000, reserve_1, reserve_0, false);
		let token_0 = ata(&trader.pubkey(), &fixture.mint_0, &fixture.program_0);
		let before = h.token_balance(&token_0);
		h.send(
			&[swap_exact_in_instruction(
				&fixture,
				&trader.pubkey(),
				false,
				200_000_000,
				out,
			)],
			&[&trader],
		)
		.expect("sell token 1");
		assert_eq!(h.token_balance(&token_0) - before, out);
		let pool = pool_state(&h, &fixture.pool);
		assert_eq!(pool.protocol_fees1, protocol_1);
		assert_eq!(pool.creator_fees0, creator + creator_0);
		assert_eq!(pool.creator_fees1, 0);

		// Buy exactly 1,000,000 token 0 and never pay more than an exact-input
		// trade for the same output would cost.
		let token_1_before = h.token_balance(&token_1);
		let token_0_before = h.token_balance(&token_0);
		h.send(
			&[swap_exact_out_instruction(
				&fixture,
				&trader.pubkey(),
				false,
				1_000_000,
				10_000_000,
			)],
			&[&trader],
		)
		.expect("buy token 0");
		assert_eq!(h.token_balance(&token_0) - token_0_before, 1_000_000);
		let paid = token_1_before - h.token_balance(&token_1);
		assert!(paid > 0 && paid <= 10_000_000);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn swaps_enforce_slippage_limits_and_pool_vaults() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let trader = funded_trader(&h, &fixture, 1_000_000_000);
		let (reserve_0, reserve_1) = reserves(&h, &fixture);
		let (out, ..) = expected_exact_in(1_000_000, reserve_0, reserve_1, true);

		h.expect_custom_error(
			&[swap_exact_in_instruction(
				&fixture,
				&trader.pubkey(),
				true,
				1_000_000,
				out + 1,
			)],
			&[&trader],
			code::SLIPPAGE_EXCEEDED,
		);
		h.expect_custom_error(
			&[swap_exact_out_instruction(
				&fixture,
				&trader.pubkey(),
				true,
				out,
				1_000_000 - 1,
			)],
			&[&trader],
			code::SLIPPAGE_EXCEEDED,
		);
		h.expect_custom_error(
			&[swap_exact_out_instruction(
				&fixture,
				&trader.pubkey(),
				true,
				reserve_1,
				u64::MAX,
			)],
			&[&trader],
			code::INSUFFICIENT_LIQUIDITY,
		);

		let mut foreign = swap_exact_in_instruction(&fixture, &trader.pubkey(), true, 1_000_000, 0);
		// Point the output vault at a token account the pool does not own.
		foreign.accounts[5].pubkey = fixture.owner_token_1;
		h.expect_custom_error(&[foreign], &[&trader], code::POOL_ACCOUNT_MISMATCH);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn liquidity_moves_in_proportion_and_the_minimum_stays_locked() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let lp = h.funded_keypair().expect("lp");
		h.mint_to_owner(
			&fixture.mint_0,
			&lp.pubkey(),
			&fixture.program_0,
			1_000_000_000,
		)
		.expect("fund 0");
		h.mint_to_owner(
			&fixture.mint_1,
			&lp.pubkey(),
			&fixture.program_1,
			4_000_000_000,
		)
		.expect("fund 1");
		h.create_ata(&lp.pubkey(), &fixture.lp_mint, &TOKEN_PROGRAM)
			.expect("lp account");

		// 10% of the supply needs 10% of each reserve, rounded up.
		h.expect_custom_error(
			&[deposit_instruction(
				&fixture,
				&lp.pubkey(),
				200_000_000,
				(99_999_999, u64::MAX),
			)],
			&[&lp],
			code::SLIPPAGE_EXCEEDED,
		);
		h.send(
			&[deposit_instruction(
				&fixture,
				&lp.pubkey(),
				200_000_000,
				(100_000_000, 400_000_000),
			)],
			&[&lp],
		)
		.expect("deposit");
		let (_, _, lp_account) = liquidity_accounts(&fixture, &lp.pubkey());
		assert_eq!(h.token_balance(&lp_account), 200_000_000);
		assert_eq!(pool_state(&h, &fixture.pool).lp_supply, 2_200_000_000);

		h.send(
			&[withdraw_instruction(
				&fixture,
				&lp.pubkey(),
				200_000_000,
				(100_000_000, 400_000_000),
			)],
			&[&lp],
		)
		.expect("withdraw");
		assert_eq!(h.token_balance(&lp_account), 0);

		// The creator can withdraw everything they hold, but never the locked
		// minimum, so the pool keeps both reserves.
		let held = h.token_balance(&fixture.owner_lp);
		h.send(
			&[withdraw_instruction(
				&fixture,
				&fixture.owner.pubkey(),
				held,
				(0, 0),
			)],
			&[&fixture.owner],
		)
		.expect("withdraw everything");
		let pool = pool_state(&h, &fixture.pool);
		assert_eq!(pool.lp_supply, MINIMUM_LIQUIDITY);
		let (reserve_0, reserve_1) = reserves(&h, &fixture);
		assert!(reserve_0 > 0 && reserve_1 > 0);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn donations_accrue_to_liquidity_providers() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let donor = funded_trader(&h, &fixture, 1_000_000_000);
		let donor_token = ata(&donor.pubkey(), &fixture.mint_0, &fixture.program_0);
		// A plain transfer is the simplest way to model an unsolicited donation.
		#[allow(deprecated)]
		let donation = spl_token_2022_interface::instruction::transfer(
			&fixture.program_0,
			&donor_token,
			&fixture.vault_0,
			&donor.pubkey(),
			&[],
			500_000_000,
		)
		.expect("donation");
		h.send(&[donation], &[&donor]).expect("donate");

		let held = h.token_balance(&fixture.owner_lp);
		let before = h.token_balance(&fixture.owner_token_0);
		h.send(
			&[withdraw_instruction(
				&fixture,
				&fixture.owner.pubkey(),
				held,
				(0, 0),
			)],
			&[&fixture.owner],
		)
		.expect("withdraw");
		let received = h.token_balance(&fixture.owner_token_0) - before;
		// The owner holds all but the locked minimum of the supply, so they
		// receive nearly all of the 1.5e9 token 0 reserve.
		assert!(received > 1_499_000_000, "received {received}");
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn fees_are_collected_only_by_their_owners() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let trader = funded_trader(&h, &fixture, 1_000_000_000);
		for zero_for_one in [true, false] {
			h.send(
				&[swap_exact_in_instruction(
					&fixture,
					&trader.pubkey(),
					zero_for_one,
					100_000_000,
					0,
				)],
				&[&trader],
			)
			.expect("swap");
		}
		let pool = pool_state(&h, &fixture.pool);
		assert!(pool.protocol_fees0 > 0 && pool.protocol_fees1 > 0);
		assert!(pool.creator_fees0 > 0 && pool.creator_fees1 > 0);

		let recipient = h.funded_keypair().expect("recipient");
		let recipient_0 = h
			.create_ata(&recipient.pubkey(), &fixture.mint_0, &fixture.program_0)
			.expect("recipient 0");
		let recipient_1 = h
			.create_ata(&recipient.pubkey(), &fixture.mint_1, &fixture.program_1)
			.expect("recipient 1");
		let protocol = |authority: &Pubkey| {
			CollectProtocolFees::new(
				*authority,
				fixture.config,
				fixture.pool,
				fixture.vault_0,
				fixture.vault_1,
				recipient_0,
				recipient_1,
				fixture.program_0,
				fixture.program_1,
			)
			.instruction(
				CollectProtocolFeesInstructionData::new(|data| {
					data.maximum_amount0.set(u64::MAX);
					data.maximum_amount1.set(u64::MAX);
				})
				.expect("collect data"),
			)
		};
		let creator = |authority: &Pubkey| {
			CollectCreatorFees::new(
				*authority,
				fixture.pool,
				fixture.vault_0,
				fixture.vault_1,
				recipient_0,
				recipient_1,
				fixture.program_0,
				fixture.program_1,
			)
			.instruction(
				CollectCreatorFeesInstructionData::new(|data| {
					data.maximum_amount0.set(u64::MAX);
					data.maximum_amount1.set(u64::MAX);
				})
				.expect("collect data"),
			)
		};

		h.expect_custom_error(
			&[protocol(&trader.pubkey())],
			&[&trader],
			code::UNAUTHORIZED,
		);
		h.expect_custom_error(&[creator(&trader.pubkey())], &[&trader], code::UNAUTHORIZED);

		let reserves_before = reserves(&h, &fixture);
		h.send(&[protocol(&admin.pubkey())], &[&admin])
			.expect("collect protocol");
		h.send(&[creator(&fixture.owner.pubkey())], &[&fixture.owner])
			.expect("collect creator");
		assert_eq!(
			h.token_balance(&recipient_0),
			pool.protocol_fees0 + pool.creator_fees0
		);
		assert_eq!(
			h.token_balance(&recipient_1),
			pool.protocol_fees1 + pool.creator_fees1
		);
		let collected = pool_state(&h, &fixture.pool);
		assert_eq!(
			(
				collected.protocol_fees0,
				collected.protocol_fees1,
				collected.creator_fees0,
				collected.creator_fees1
			),
			(0, 0, 0, 0)
		);
		assert_eq!(
			reserves(&h, &fixture),
			reserves_before,
			"collection never touches liquidity"
		);

		// A config that is not the pool's tier is rejected.
		let other_config = create_config(&h, &admin, 1, &Pubkey::default());
		let mut wrong_tier = protocol(&admin.pubkey());
		wrong_tier.accounts[1].pubkey = other_config;
		h.expect_custom_error(&[wrong_tier], &[&admin], code::POOL_ACCOUNT_MISMATCH);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn set_pool_creator_hands_over_fee_rights() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let successor = h.funded_keypair().expect("successor");
		let set_creator = |signer: &Pubkey, new_creator: &Pubkey| {
			SetPoolCreator::new(*signer, fixture.pool).instruction(
				SetPoolCreatorInstructionData::new(|data| data.new_creator = *new_creator)
					.expect("set creator data"),
			)
		};
		h.expect_custom_error(
			&[set_creator(&successor.pubkey(), &successor.pubkey())],
			&[&successor],
			code::UNAUTHORIZED,
		);
		h.send(
			&[set_creator(&fixture.owner.pubkey(), &successor.pubkey())],
			&[&fixture.owner],
		)
		.expect("set creator");
		assert_eq!(pool_state(&h, &fixture.pool).creator, successor.pubkey());
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn restricted_tiers_only_accept_their_pool_creator() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let launchpad = h.funded_keypair().expect("launchpad");
		let config = create_config(&h, &admin, 7, &launchpad.pubkey());
		let (token_0, token_1) = ordered_mints(&h, &TOKEN_PROGRAM, &[], &TOKEN_PROGRAM, &[]);
		let owner = h.funded_keypair().expect("owner");
		let spec = |authority| {
			PoolSpec {
				config,
				token_0,
				token_1,
				amounts: (1_000_000_000, 1_000_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: authority,
			}
		};

		let (without, _) = create_pool_instruction(&h, &spec(None), &owner);
		h.expect_custom_error(&[without], &[&owner], code::POOL_CREATOR_NOT_AUTHORIZED);
		let (impostor, _) = create_pool_instruction(&h, &spec(Some(&owner)), &owner);
		h.expect_custom_error(&[impostor], &[&owner], code::POOL_CREATOR_NOT_AUTHORIZED);

		let (allowed, fixture) = create_pool_instruction(&h, &spec(Some(&launchpad)), &owner);
		h.send(&[allowed], &[&owner, &launchpad])
			.expect("create pool");
		assert_eq!(pool_state(&h, &fixture.pool).amm_config, config);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `pina test`"]
fn prefunded_pool_addresses_cannot_block_creation() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let config = create_config(&h, &admin, 0, &Pubkey::default());
		let (token_0, token_1) = ordered_mints(&h, &TOKEN_PROGRAM, &[], &TOKEN_PROGRAM, &[]);
		let pool = Pool::find_pda(&config, &token_0.0, &token_1.0).0;
		for address in [
			pool,
			vault_address(&pool, &token_0.0),
			vault_address(&pool, &token_1.0),
			lp_mint_address(&pool),
		] {
			h.fund(&address, 1_000_000).expect("prefund");
		}
		let fixture = create_pool(
			&h,
			&PoolSpec {
				config,
				token_0,
				token_1,
				amounts: (1_000_000_000, 1_000_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: None,
			},
		);
		assert_eq!(h.token_balance(&fixture.vault_0), 1_000_000_000);
		h.stop().expect("stop");
	});
}

/// Compute-unit ceilings per instruction. Steady-state instructions sit about
/// 20% above their measured cost so a regression fails here before it reaches
/// users. Pool creation searches for four PDA bumps whose cost varies with the
/// addresses involved, so its ceiling leaves room for that spread.
#[test]
#[ignore = "run with `pina test`"]
fn compute_units_stay_within_budget() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 1);
		let trader = funded_trader(&h, &fixture, 1_000_000_000);
		let lp = h.funded_keypair().expect("lp");
		h.mint_to_owner(
			&fixture.mint_0,
			&lp.pubkey(),
			&fixture.program_0,
			1_000_000_000,
		)
		.expect("fund 0");
		h.mint_to_owner(
			&fixture.mint_1,
			&lp.pubkey(),
			&fixture.program_1,
			4_000_000_000,
		)
		.expect("fund 1");
		h.create_ata(&lp.pubkey(), &fixture.lp_mint, &TOKEN_PROGRAM)
			.expect("lp account");

		let measure = |label: &str,
		               instruction: Instruction,
		               signers: &[&dyn Signer],
		               budget: u64| {
			let units = h
				.simulate(&[instruction], signers)
				.unwrap_or_else(|(error, logs)| panic!("{label}: {error:?}\n{}", logs.join("\n")));
			println!("{label}: {units} CU (budget {budget})");
			assert!(
				units <= budget,
				"{label} used {units} CU, above its {budget} CU budget"
			);
		};
		measure(
			"swap_exact_in",
			swap_exact_in_instruction(&fixture, &trader.pubkey(), true, 1_000_000, 0),
			&[&trader],
			6_000,
		);
		measure(
			"swap_exact_out",
			swap_exact_out_instruction(&fixture, &trader.pubkey(), false, 1_000_000, u64::MAX),
			&[&trader],
			6_000,
		);
		measure(
			"deposit",
			deposit_instruction(&fixture, &lp.pubkey(), 1_000_000, (u64::MAX, u64::MAX)),
			&[&lp],
			6_500,
		);
		measure(
			"withdraw",
			withdraw_instruction(&fixture, &fixture.owner.pubkey(), 1_000_000, (0, 0)),
			&[&fixture.owner],
			6_500,
		);

		let config = create_config(&h, &admin, 1, &Pubkey::default());
		let (token_0, token_1) = ordered_mints(&h, &TOKEN_PROGRAM, &[], &TOKEN_PROGRAM, &[]);
		let owner = h.funded_keypair().expect("owner");
		let (create, _) = create_pool_instruction(
			&h,
			&PoolSpec {
				config,
				token_0,
				token_1,
				amounts: (1_000_000_000, 1_000_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: None,
			},
			&owner,
		);
		measure("create_pool", create, &[&owner], 60_000);
		h.stop().expect("stop");
	});
}
