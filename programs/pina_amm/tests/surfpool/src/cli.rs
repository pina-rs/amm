//! Drives the compiled `pina-amm` binary against a live Surfnet.
//!
//! Build the CLI first (`cargo build -p pina_amm_cli` from the repository
//! root); `devenv shell test:surfpool` does this before running the suite.

use std::path::PathBuf;
use std::process::Command;

use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;

use crate::harness::Harness;
use crate::harness::TOKEN_PROGRAM;
use crate::harness::ata;

fn binary() -> PathBuf {
	std::env::var_os("PINA_AMM_CLI").map_or_else(
		|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../target/debug/pina-amm"),
		PathBuf::from,
	)
}

/// Run `pina-amm --json <args>` as `signer` and parse its JSON output.
fn run(h: &Harness, signer: &Keypair, args: &[&str]) -> serde_json::Value {
	let directory = std::env::temp_dir().join(format!("pina-amm-cli-{}", signer.pubkey()));
	std::fs::create_dir_all(&directory).expect("keypair directory");
	let keypair = directory.join("signer.json");
	std::fs::write(
		&keypair,
		serde_json::to_string(&signer.to_bytes().to_vec()).expect("keypair json"),
	)
	.expect("write keypair");
	let output = Command::new(binary())
		.arg("--url")
		.arg(h.rpc_url())
		.arg("--keypair")
		.arg(&keypair)
		.arg("--json")
		.args(args)
		.output()
		.unwrap_or_else(|error| {
			panic!(
				"run {}: {error}; build it with `cargo build -p pina_amm_cli`",
				binary().display()
			)
		});
	assert!(
		output.status.success(),
		"pina-amm {args:?} failed:\n{}",
		String::from_utf8_lossy(&output.stderr)
	);
	serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
		panic!(
			"pina-amm {args:?} printed invalid JSON ({error}):\n{}",
			String::from_utf8_lossy(&output.stdout)
		)
	})
}

#[test]
#[ignore = "run with `pina test`"]
fn the_cli_runs_a_complete_pool_lifecycle() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = h.funded_keypair().expect("admin");
		h.set_upgrade_authority(&admin.pubkey()).expect("authority");
		let mint_a = h.create_mint(&TOKEN_PROGRAM, 6, &[]).expect("mint a");
		let mint_b = h.create_mint(&TOKEN_PROGRAM, 6, &[]).expect("mint b");
		h.mint_to_owner(&mint_a, &admin.pubkey(), &TOKEN_PROGRAM, 10_000_000_000)
			.expect("fund a");
		h.mint_to_owner(&mint_b, &admin.pubkey(), &TOKEN_PROGRAM, 10_000_000_000)
			.expect("fund b");

		let created = run(
			&h,
			&admin,
			&[
				"config",
				"create",
				"--index",
				"3",
				"--trade-fee-rate",
				"3000",
				"--protocol-fee-rate",
				"100000",
			],
		);
		assert_eq!(created["action"], "config created");
		let shown = run(&h, &admin, &["config", "show", "--index", "3"]);
		assert_eq!(shown["trade_fee_rate"], 3000);
		assert_eq!(shown["authority"], admin.pubkey().to_string());

		let pool = run(
			&h,
			&admin,
			&[
				"pool",
				"create",
				"--config",
				"3",
				"--mint-a",
				&mint_a.to_string(),
				"--mint-b",
				&mint_b.to_string(),
				"--amount-a",
				"1000000000",
				"--amount-b",
				"2000000000",
			],
		);
		let address = pool["details"]["pool"]
			.as_str()
			.expect("pool address")
			.to_owned();

		let quote = run(
			&h,
			&admin,
			&[
				"quote",
				"--pool",
				&address,
				"--sell",
				&mint_a.to_string(),
				"--amount-in",
				"1000000",
			],
		);
		let quoted_out = quote["details"]["amount_out"].as_u64().expect("quoted out");
		assert!(quoted_out > 0);

		let token_b = ata(&admin.pubkey(), &mint_b, &TOKEN_PROGRAM);
		let before = h.token_balance(&token_b);
		run(
			&h,
			&admin,
			&[
				"swap",
				"--pool",
				&address,
				"--sell",
				&mint_a.to_string(),
				"--amount-in",
				"1000000",
			],
		);
		assert_eq!(h.token_balance(&token_b) - before, quoted_out);

		run(
			&h,
			&admin,
			&[
				"swap",
				"--pool",
				&address,
				"--sell",
				&mint_b.to_string(),
				"--amount-out",
				"500000",
			],
		);
		run(
			&h,
			&admin,
			&["deposit", "--pool", &address, "--lp-amount", "1000000"],
		);
		run(
			&h,
			&admin,
			&["withdraw", "--pool", &address, "--lp-amount", "1000000"],
		);
		let protocol = run(
			&h,
			&admin,
			&["fees", "collect-protocol", "--pool", &address],
		);
		assert_eq!(protocol["action"], "protocol fees collected");
		run(&h, &admin, &["fees", "collect-creator", "--pool", &address]);

		let shown = run(&h, &admin, &["pool", "show", "--pool", &address]);
		assert_eq!(shown["protocol_fees"], serde_json::json!([0, 0]));
		assert_eq!(shown["creator_fees"], serde_json::json!([0, 0]));
		let successor = Pubkey::new_unique().to_string();
		run(
			&h,
			&admin,
			&[
				"pool",
				"set-creator",
				"--pool",
				&address,
				"--new-creator",
				&successor,
			],
		);
		let shown = run(&h, &admin, &["pool", "show", "--pool", &address]);
		assert_eq!(shown["creator"], successor);
		h.stop().expect("stop");
	});
}
