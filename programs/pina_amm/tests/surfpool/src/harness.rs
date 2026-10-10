//! A small, explicit harness around an offline Surfnet.
//!
//! The harness deploys the compiled program, builds token fixtures with the
//! real SPL Token and Token-2022 programs, and submits or simulates
//! transactions. Every helper returns a `Result` with the failing step in the
//! message so a broken journey says where it broke.

use std::path::PathBuf;

use pina_amm_client::PINA_AMM_ID;
use pina_test::Account;
use pina_test::AccountMeta;
use pina_test::Instruction;
use pina_test::InstructionError;
use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;
use pina_test::TransactionError;
use solana_commitment_config::CommitmentConfig;
use solana_message::Message;
use solana_program_pack::Pack;
use solana_rpc_client_api::config::RpcSimulateTransactionConfig;
use solana_transaction::Transaction;
use spl_token_2022_interface::extension::ExtensionType;
use spl_token_2022_interface::extension::metadata_pointer;
use spl_token_2022_interface::extension::transfer_fee;
use surfpool_sdk::Surfnet;
use surfpool_sdk::cheatcodes::builders::CheatcodeBuilder;
use surfpool_sdk::cheatcodes::builders::DeployProgram;

/// The original SPL Token program.
pub const TOKEN_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// The Token-2022 program.
pub const TOKEN_2022_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
/// The associated token account program.
pub const ATA_PROGRAM: Pubkey =
	Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
/// The system program.
pub const SYSTEM_PROGRAM: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
/// The clock sysvar.
pub const CLOCK_SYSVAR: Pubkey =
	Pubkey::from_str_const("SysvarC1ock11111111111111111111111111111111");
/// The upgradeable BPF loader.
pub const UPGRADEABLE_LOADER: Pubkey =
	Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");

/// A Token-2022 extension a test mint is created with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MintExtension {
	/// A metadata pointer, which the AMM accepts.
	MetadataPointer,
	/// A transfer fee, which the AMM rejects.
	TransferFee,
}

/// `surfnet_setProgramAuthority`, which `surfpool-sdk` does not wrap yet.
struct SetProgramAuthority {
	program_id: Pubkey,
	authority: Pubkey,
}

impl CheatcodeBuilder for SetProgramAuthority {
	const METHOD: &'static str = "surfnet_setProgramAuthority";

	fn build(self) -> serde_json::Value {
		serde_json::json!([self.program_id.to_string(), self.authority.to_string()])
	}
}

/// An offline Surfnet with the Pina AMM deployed at its program id.
pub struct Harness {
	surfnet: Surfnet,
}

impl Harness {
	/// Start an offline Surfnet and deploy the AMM artifact.
	///
	/// `pina test` sets `PINA_SBF_ARTIFACT`; a plain `cargo test` falls back to
	/// the workspace's `target/deploy/pina_amm.so`, which `pina build` writes.
	pub async fn start() -> Result<Self, String> {
		let artifact = std::env::var_os("PINA_SBF_ARTIFACT").map_or_else(
			|| {
				PathBuf::from(env!("CARGO_MANIFEST_DIR"))
					.join("../../../../target/deploy/pina_amm.so")
			},
			PathBuf::from,
		);
		if !artifact.is_file() {
			return Err(format!(
				"missing SBF artifact {}; run `pina build` first",
				artifact.display()
			));
		}
		let surfnet = Surfnet::builder()
			.offline(true)
			.slot_time_ms(400)
			.airdrop_sol(1_000_000_000_000)
			.start()
			.await
			.map_err(|error| format!("start offline Surfpool: {error}"))?;
		surfnet
			.cheatcodes()
			.deploy(DeployProgram::new(PINA_AMM_ID).so_path(artifact))
			.map_err(|error| format!("deploy program: {error}"))?;
		Ok(Self { surfnet })
	}

	/// The Surfnet's JSON-RPC URL.
	pub fn rpc_url(&self) -> &str {
		self.surfnet.rpc_url()
	}

	/// The pre-funded fee payer.
	pub fn payer(&self) -> &Keypair {
		self.surfnet.payer()
	}

	/// Make `authority` the AMM's upgrade authority.
	pub fn set_upgrade_authority(&self, authority: &Pubkey) -> Result<(), String> {
		self.surfnet
			.cheatcodes()
			.execute(SetProgramAuthority {
				program_id: PINA_AMM_ID,
				authority: *authority,
			})
			.map_err(|error| format!("set upgrade authority: {error}"))
	}

	/// The cluster's Unix timestamp, read from the clock sysvar.
	pub fn now(&self) -> i64 {
		let clock = self.account(&CLOCK_SYSVAR).expect("clock sysvar");
		i64::from_le_bytes(clock.data[32..40].try_into().expect("unix timestamp"))
	}

	/// Move the cluster clock forward by `seconds`.
	pub fn advance_seconds(&self, seconds: i64) -> Result<(), String> {
		let target = u64::try_from(self.now() + seconds).map_err(|error| error.to_string())?;
		let _ = self.surfnet.events().try_iter().count();
		self.surfnet
			.cheatcodes()
			.time_travel_to_timestamp(target * 1_000)
			.map(|_| ())
			.map_err(|error| format!("time travel: {error}"))
	}

	/// The AMM's program-data account.
	pub fn program_data() -> Pubkey {
		Pubkey::find_program_address(&[PINA_AMM_ID.as_ref()], &UPGRADEABLE_LOADER).0
	}

	fn signed_transaction(
		&self,
		instructions: &[Instruction],
		signers: &[&dyn Signer],
	) -> Result<Transaction, String> {
		// Surfpool's observer channel is bounded; drain it so long journeys
		// never stall on unread events.
		let _ = self.surfnet.events().try_iter().count();
		let rpc = self.surfnet.rpc_client();
		let payer = self.surfnet.payer();
		let mut all_signers: Vec<&dyn Signer> = vec![payer];
		all_signers.extend_from_slice(signers);
		let blockhash = rpc
			.get_latest_blockhash()
			.map_err(|error| format!("fetch blockhash: {error}"))?;
		let message = Message::new(instructions, Some(&payer.pubkey()));
		let mut transaction = Transaction::new_unsigned(message);
		transaction
			.try_sign(&all_signers, blockhash)
			.map_err(|error| format!("sign transaction: {error}"))?;
		Ok(transaction)
	}

	/// Submit and confirm a transaction paid for by the harness payer.
	pub fn send(
		&self,
		instructions: &[Instruction],
		signers: &[&dyn Signer],
	) -> Result<(), String> {
		let transaction = self.signed_transaction(instructions, signers)?;
		self.surfnet
			.rpc_client()
			.send_and_confirm_transaction_with_spinner_and_commitment(
				&transaction,
				CommitmentConfig::processed(),
			)
			.map(|_| ())
			.map_err(|error| format!("execute transaction: {error}"))
	}

	/// Simulate a transaction and return its compute units, or its error and
	/// logs when it fails.
	pub fn simulate(
		&self,
		instructions: &[Instruction],
		signers: &[&dyn Signer],
	) -> Result<u64, (Option<TransactionError>, Vec<String>)> {
		let transaction = self
			.signed_transaction(instructions, signers)
			.map_err(|error| (None, vec![error]))?;
		let result = self
			.surfnet
			.rpc_client()
			.simulate_transaction_with_config(
				&transaction,
				RpcSimulateTransactionConfig {
					commitment: Some(CommitmentConfig::processed()),
					..RpcSimulateTransactionConfig::default()
				},
			)
			.map_err(|error| (None, vec![format!("simulate transaction: {error}")]))?
			.value;
		let logs = result.logs.unwrap_or_default();
		match result.err {
			Some(error) => Err((Some(error.into()), logs)),
			None => Ok(result.units_consumed.unwrap_or_default()),
		}
	}

	/// Require a transaction to fail with the AMM's custom error `code` in its
	/// last instruction.
	pub fn expect_custom_error(
		&self,
		instructions: &[Instruction],
		signers: &[&dyn Signer],
		code: u32,
	) {
		match self.simulate(instructions, signers) {
			Ok(_) => panic!("expected custom error {code}, but the transaction succeeded"),
			Err((
				Some(TransactionError::InstructionError(_, InstructionError::Custom(actual))),
				logs,
			)) => {
				assert_eq!(
					actual,
					code,
					"unexpected custom error; logs:\n{}",
					logs.join("\n")
				);
			}
			Err((error, logs)) => {
				panic!(
					"expected custom error {code}, got {error:?}; logs:\n{}",
					logs.join("\n")
				)
			}
		}
	}

	/// Fetch an account, if it exists.
	pub fn account(&self, address: &Pubkey) -> Option<Account> {
		self.surfnet.rpc_client().get_account(address).ok()
	}

	/// Send `lamports` from the payer to `address`.
	pub fn fund(&self, address: &Pubkey, lamports: u64) -> Result<(), String> {
		self.send(
			&[system_transfer(&self.payer().pubkey(), address, lamports)],
			&[],
		)
	}

	/// A new keypair holding enough SOL for rent and fees.
	pub fn funded_keypair(&self) -> Result<Keypair, String> {
		let keypair = Keypair::new();
		self.fund(&keypair.pubkey(), 1_000_000_000)?;
		Ok(keypair)
	}

	/// Create a mint with `decimals`, owned by `token_program`, whose mint
	/// authority is the harness payer.
	pub fn create_mint(
		&self,
		token_program: &Pubkey,
		decimals: u8,
		extensions: &[MintExtension],
	) -> Result<Pubkey, String> {
		let mint = Keypair::new();
		let payer = self.payer().pubkey();
		let extension_types = extensions
			.iter()
			.map(|extension| {
				match extension {
					MintExtension::MetadataPointer => ExtensionType::MetadataPointer,
					MintExtension::TransferFee => ExtensionType::TransferFeeConfig,
				}
			})
			.collect::<Vec<_>>();
		let space = if extension_types.is_empty() {
			spl_token_2022_interface::state::Mint::LEN
		} else {
			ExtensionType::try_calculate_account_len::<spl_token_2022_interface::state::Mint>(
				&extension_types,
			)
			.map_err(|error| format!("size mint: {error}"))?
		};
		let lamports = self
			.surfnet
			.rpc_client()
			.get_minimum_balance_for_rent_exemption(space)
			.map_err(|error| format!("rent: {error}"))?;
		let mut instructions = vec![system_create_account(
			&payer,
			&mint.pubkey(),
			lamports,
			space as u64,
			token_program,
		)];
		for extension in extensions {
			instructions.push(match extension {
				MintExtension::MetadataPointer => {
					metadata_pointer::instruction::initialize(
						token_program,
						&mint.pubkey(),
						Some(payer),
						Some(mint.pubkey()),
					)
					.map_err(|error| format!("metadata pointer: {error}"))?
				}
				MintExtension::TransferFee => {
					transfer_fee::instruction::initialize_transfer_fee_config(
						token_program,
						&mint.pubkey(),
						Some(&payer),
						Some(&payer),
						100,
						1_000_000,
					)
					.map_err(|error| format!("transfer fee: {error}"))?
				}
			});
		}
		instructions.push(
			spl_token_2022_interface::instruction::initialize_mint2(
				token_program,
				&mint.pubkey(),
				&payer,
				None,
				decimals,
			)
			.map_err(|error| format!("initialize mint: {error}"))?,
		);
		self.send(&instructions, &[&mint])?;
		Ok(mint.pubkey())
	}

	/// Create `owner`'s associated token account for `mint` if it is missing
	/// and return its address.
	pub fn create_ata(
		&self,
		owner: &Pubkey,
		mint: &Pubkey,
		token_program: &Pubkey,
	) -> Result<Pubkey, String> {
		let address = ata(owner, mint, token_program);
		let payer = self.payer().pubkey();
		self.send(
			&[Instruction::new_with_bytes(
				ATA_PROGRAM,
				&[1],
				vec![
					AccountMeta::new(payer, true),
					AccountMeta::new(address, false),
					AccountMeta::new_readonly(*owner, false),
					AccountMeta::new_readonly(*mint, false),
					AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
					AccountMeta::new_readonly(*token_program, false),
				],
			)],
			&[],
		)?;
		Ok(address)
	}

	/// Mint `amount` to `owner`'s associated token account, creating it first.
	pub fn mint_to_owner(
		&self,
		mint: &Pubkey,
		owner: &Pubkey,
		token_program: &Pubkey,
		amount: u64,
	) -> Result<Pubkey, String> {
		let account = self.create_ata(owner, mint, token_program)?;
		let instruction = spl_token_2022_interface::instruction::mint_to(
			token_program,
			mint,
			&account,
			&self.payer().pubkey(),
			&[],
			amount,
		)
		.map_err(|error| format!("mint to: {error}"))?;
		self.send(&[instruction], &[])?;
		Ok(account)
	}

	/// Token balance of a token account; zero when the account is missing.
	pub fn token_balance(&self, account: &Pubkey) -> u64 {
		self.account(account).map_or(0, |account| {
			spl_token_2022_interface::state::Account::unpack_from_slice(
				&account.data[..spl_token_2022_interface::state::Account::LEN],
			)
			.map_or(0, |state| state.amount)
		})
	}

	/// Supply of a mint.
	pub fn mint_supply(&self, mint: &Pubkey) -> u64 {
		self.account(mint).map_or(0, |account| {
			spl_token_2022_interface::state::Mint::unpack_from_slice(
				&account.data[..spl_token_2022_interface::state::Mint::LEN],
			)
			.map_or(0, |state| state.supply)
		})
	}

	/// Stop the Surfnet and release its ports.
	pub fn stop(mut self) -> Result<(), String> {
		self.surfnet
			.stop()
			.map_err(|error| format!("stop Surfpool: {error}"))
	}
}

/// The associated token account of `owner` for `mint`.
pub fn ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
		&ATA_PROGRAM,
	)
	.0
}

/// A system-program transfer.
pub fn system_transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
	let mut data = 2u32.to_le_bytes().to_vec();
	data.extend_from_slice(&lamports.to_le_bytes());
	Instruction::new_with_bytes(
		SYSTEM_PROGRAM,
		&data,
		vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)],
	)
}

/// A system-program account creation.
pub fn system_create_account(
	from: &Pubkey,
	to: &Pubkey,
	lamports: u64,
	space: u64,
	owner: &Pubkey,
) -> Instruction {
	let mut data = 0u32.to_le_bytes().to_vec();
	data.extend_from_slice(&lamports.to_le_bytes());
	data.extend_from_slice(&space.to_le_bytes());
	data.extend_from_slice(owner.as_ref());
	Instruction::new_with_bytes(
		SYSTEM_PROGRAM,
		&data,
		vec![AccountMeta::new(*from, true), AccountMeta::new(*to, true)],
	)
}
