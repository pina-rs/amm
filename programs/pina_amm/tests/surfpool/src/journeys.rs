//! Every instruction, in one ordered journey, through every client that can
//! talk to a live Surfnet.
//!
//! The Rust journey drives all ten instructions through the generated Rust
//! client in a single lifecycle, which also gives the performance benchmark
//! one deterministic sample per instruction. The TypeScript journey drives
//! the same lifecycle through the generated `@pina-rs/amm` client: the Rust
//! side prepares token fixtures, funds the signers, and derives every
//! address, then asserts the resulting on-chain state, so the wire format
//! the TypeScript client produces is proven end to end.
//!
//! The Dart client is codecs-only by design (it ships no RPC or transaction
//! dependency), so its surface stays covered by its unit tests; the CLI
//! already drives the program end to end in [`crate::cli`].

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use pina_amm_client::instructions::CollectCreatorFees;
use pina_amm_client::instructions::CollectCreatorFeesInstructionData;
use pina_amm_client::instructions::CollectProtocolFees;
use pina_amm_client::instructions::CollectProtocolFeesInstructionData;
use pina_amm_client::instructions::SetPoolCreator;
use pina_amm_client::instructions::SetPoolCreatorInstructionData;
use pina_amm_client::instructions::UpdateConfig;
use pina_amm_client::instructions::UpdateConfigInstructionData;
use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;

use crate::PoolSpec;
use crate::config_address;
use crate::create_config;
use crate::create_pool_instruction;
use crate::deposit_instruction;
use crate::funded_trader;
use crate::harness::Harness;
use crate::harness::TOKEN_PROGRAM;
use crate::harness::ata;
use crate::liquidity_accounts;
use crate::pool_state;
use crate::swap_exact_in_instruction;
use crate::swap_exact_out_instruction;
use crate::withdraw_instruction;

/// The tier index every journey uses.
const JOURNEY_CONFIG_INDEX: u16 = 77;

/// Fixed seeds for every keypair that feeds a PDA derivation, so the journey
/// costs the same compute units on every run and the performance benchmark
/// compares programs, not addresses.
const ADMIN_SEED: [u8; 32] = *b"pina amm journey admin key 00010";
const OWNER_SEED: [u8; 32] = *b"pina amm journey owner key 00010";
const MINT_0_SEED: [u8; 32] = *b"pina amm journey mint zero 00100";
const MINT_1_SEED: [u8; 32] = *b"pina amm journey mint one 000100";

/// Two fresh mints with fixed addresses, ordered as the program requires.
fn journey_mints(h: &Harness) -> ((Pubkey, Pubkey), (Pubkey, Pubkey)) {
	let zero = Keypair::new_from_array(MINT_0_SEED);
	let one = Keypair::new_from_array(MINT_1_SEED);
	let a = h
		.create_mint_with_key(&zero, &TOKEN_PROGRAM, 6, &[])
		.expect("mint zero");
	let b = h
		.create_mint_with_key(&one, &TOKEN_PROGRAM, 9, &[])
		.expect("mint one");
	if a < b {
		((a, TOKEN_PROGRAM), (b, TOKEN_PROGRAM))
	} else {
		((b, TOKEN_PROGRAM), (a, TOKEN_PROGRAM))
	}
}

/// The journey's tier authority: a funded, fixed-seed keypair installed as
/// the program's upgrade authority.
fn journey_admin(h: &Harness) -> Keypair {
	let admin = Keypair::new_from_array(ADMIN_SEED);
	h.fund(&admin.pubkey(), 1_000_000_000).expect("fund admin");
	h.set_upgrade_authority(&admin.pubkey())
		.expect("set upgrade authority");
	admin
}

/// Run all ten instructions through the generated Rust client, in order, on
/// one pool, asserting the state each one leaves behind.
#[test]
#[ignore = "run with `pina test`"]
fn every_instruction_runs_in_one_journey() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = journey_admin(&h);
		let config = create_config(&h, &admin, JOURNEY_CONFIG_INDEX, &Pubkey::default());

		// UpdateConfig keeps the authority and re-states the rates.
		let update = UpdateConfig::new(admin.pubkey(), config).instruction(
			UpdateConfigInstructionData::new(|data| {
				data.new_authority = admin.pubkey();
				data.trade_fee_rate.set(2_500);
				data.protocol_fee_rate.set(200_000);
				data.creator_fee_rate.set(500);
			})
			.expect("update config data"),
		);
		h.send(&[update], &[&admin]).expect("update config");

		let (token_0, token_1) = journey_mints(&h);
		let owner = Keypair::new_from_array(OWNER_SEED);
		h.fund(&owner.pubkey(), 1_000_000_000).expect("fund owner");
		let (create, fixture) = create_pool_instruction(
			&h,
			&PoolSpec {
				config,
				token_0,
				token_1,
				amounts: (1_000_000_000, 4_000_000_000),
				creator_fee_mode: 0,
				pool_creator_authority: None,
			},
			&owner,
		);
		h.send(&[create], &[&owner]).expect("create pool");
		assert_eq!(pool_state(&h, &fixture.pool).lp_supply, 2_000_000_000);

		h.send(
			&[deposit_instruction(
				&fixture,
				&owner.pubkey(),
				1_000_000,
				(u64::MAX, u64::MAX),
			)],
			&[&owner],
		)
		.expect("deposit");
		assert_eq!(pool_state(&h, &fixture.pool).lp_supply, 2_001_000_000);

		let trader = funded_trader(&h, &fixture, 1_000_000_000);
		h.send(
			&[swap_exact_in_instruction(
				&fixture,
				&trader.pubkey(),
				true,
				10_000_000,
				0,
			)],
			&[&trader],
		)
		.expect("swap exact in");
		h.send(
			&[swap_exact_out_instruction(
				&fixture,
				&trader.pubkey(),
				true,
				1_000_000,
				u64::MAX,
			)],
			&[&trader],
		)
		.expect("swap exact out");
		let pool = pool_state(&h, &fixture.pool);
		assert!(pool.protocol_fees0 > 0, "swaps must accrue fees");

		let (recipient_0, recipient_1, _) = liquidity_accounts(&fixture, &owner.pubkey());
		let collect_creator = CollectCreatorFees::new(
			owner.pubkey(),
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
			.expect("collect creator data"),
		);
		h.send(&[collect_creator], &[&owner])
			.expect("collect creator fees");
		let collected = pool_state(&h, &fixture.pool);
		assert_eq!(collected.creator_fees0, 0, "creator fees were paid");

		let collect_protocol = CollectProtocolFees::new(
			admin.pubkey(),
			config,
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
			.expect("collect protocol data"),
		);
		h.send(&[collect_protocol], &[&admin])
			.expect("collect protocol fees");
		assert_eq!(pool_state(&h, &fixture.pool).protocol_fees0, 0);

		let successor = h.funded_keypair().expect("successor");
		let set_creator = SetPoolCreator::new(owner.pubkey(), fixture.pool).instruction(
			SetPoolCreatorInstructionData::new(|data| data.new_creator = successor.pubkey())
				.expect("set creator data"),
		);
		h.send(&[set_creator], &[&owner]).expect("set pool creator");
		assert_eq!(pool_state(&h, &fixture.pool).creator, successor.pubkey());

		h.send(
			&[withdraw_instruction(
				&fixture,
				&owner.pubkey(),
				1_000_000,
				(0, 0),
			)],
			&[&owner],
		)
		.expect("withdraw");
		assert_eq!(pool_state(&h, &fixture.pool).lp_supply, 2_000_000_000);
		h.stop().expect("stop");
	});
}

/// Write the fixtures the TypeScript journey needs: funded keypairs, token
/// accounts, and every address the client should derive itself.
fn write_fixtures(path: &std::path::Path, journey: &serde_json::Value) {
	fs::write(path, serde_json::to_string(journey).expect("fixtures json"))
		.expect("write fixtures");
}

/// Run all ten instructions through the generated TypeScript client against
/// one live Surfnet, then check the state it left behind from Rust.
#[test]
#[ignore = "run with `pina test`"]
fn typescript_client_runs_every_instruction() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = journey_admin(&h);
		let owner = Keypair::new_from_array(OWNER_SEED);
		h.fund(&owner.pubkey(), 1_000_000_000).expect("fund owner");
		let ((mint_0, _), (mint_1, _)) = journey_mints(&h);
		let owner_token_0 = h
			.mint_to_owner(&mint_0, &owner.pubkey(), &TOKEN_PROGRAM, 10_000_000_000)
			.expect("fund token 0");
		let owner_token_1 = h
			.mint_to_owner(&mint_1, &owner.pubkey(), &TOKEN_PROGRAM, 40_000_000_000)
			.expect("fund token 1");

		let config = config_address(JOURNEY_CONFIG_INDEX);
		let pool = pina_amm_client::accounts::Pool::find_pda(&config, &mint_0, &mint_1).0;
		let vault_0 = crate::vault_address(&pool, &mint_0);
		let vault_1 = crate::vault_address(&pool, &mint_1);
		let lp_mint = crate::lp_mint_address(&pool);
		let owner_lp_token = ata(&owner.pubkey(), &lp_mint, &TOKEN_PROGRAM);

		let directory = std::env::temp_dir().join(format!("pina-amm-journey-{pool}"));
		fs::create_dir_all(&directory).expect("journey directory");
		let fixtures = directory.join("fixtures.json");
		let journey = serde_json::json!({
			"programData": Harness::program_data().to_string(),
			"adminKey": admin.to_bytes().to_vec(),
			"ownerKey": owner.to_bytes().to_vec(),
			"configIndex": JOURNEY_CONFIG_INDEX,
			"mint0": mint_0.to_string(),
			"mint1": mint_1.to_string(),
			"ownerToken0": owner_token_0.to_string(),
			"ownerToken1": owner_token_1.to_string(),
			"ownerLpToken": owner_lp_token.to_string(),
			"addresses": {
				"config": config.to_string(),
				"pool": pool.to_string(),
				"lpMint": lp_mint.to_string(),
				"vault0": vault_0.to_string(),
				"vault1": vault_1.to_string(),
			},
		});
		write_fixtures(&fixtures, &journey);

		// `.../programs/pina_amm/tests/surfpool` -> the repository root.
		let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.ancestors()
			.nth(4)
			.expect("repository root")
			.to_path_buf();
		let output = Command::new("pnpm")
			.current_dir(&repository)
			.args([
				"--dir",
				"clients/typescript/pina_amm",
				"run",
				"journey",
				"--",
				"--rpc",
			])
			.arg(h.rpc_url())
			.arg("--fixtures")
			.arg(&fixtures)
			.output()
			.unwrap_or_else(|error| panic!("run the TypeScript journey: {error}"));
		assert!(
			output.status.success(),
			"the TypeScript journey failed:\n{}\n{}",
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr),
		);

		let state = pool_state(&h, &pool);
		assert!(state.lp_supply > 0, "the journey created the pool");
		assert_eq!(
			state.protocol_fees0, 0,
			"the journey collected the protocol fees"
		);
		let successor_from_logs = state.creator;
		assert_ne!(successor_from_logs, owner.pubkey());
		h.stop().expect("stop");
	});
}
