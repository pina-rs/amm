//! RPC access, signing, simulation, and event extraction.

use base64::Engine;
use pina_amm_client::PINA_AMM_ID;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_rpc_client_api::config::RpcSimulateTransactionConfig;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::cli::GlobalArgs;
use crate::error::CliError;

const MAINNET: &str = "https://api.mainnet-beta.solana.com";
const DEVNET: &str = "https://api.devnet.solana.com";
const TESTNET: &str = "https://api.testnet.solana.com";
const LOCALHOST: &str = "http://127.0.0.1:8899";

/// An account fetched over RPC.
pub struct FetchedAccount {
	/// The program that owns the account.
	pub owner: Pubkey,
	/// The account's data.
	pub data: Vec<u8>,
}

/// The result of a successful simulation.
pub struct Simulation {
	/// Compute units consumed.
	pub units: u64,
	/// Every `Program data:` record the AMM itself emitted, in order.
	pub events: Vec<Vec<u8>>,
}

/// Everything a command needs to read the chain and sign for the user.
pub struct Context {
	rpc: RpcClient,
	signer: Keypair,
	/// Print JSON instead of text.
	pub json: bool,
	/// Simulate instead of sending.
	pub simulate_only: bool,
}

impl Context {
	/// Resolve the endpoint and load the signer.
	pub fn new(args: &GlobalArgs) -> Result<Self, CliError> {
		Ok(Self {
			rpc: RpcClient::new_with_commitment(
				resolve_endpoint(&args.url)?,
				CommitmentConfig::confirmed(),
			),
			signer: load_keypair(&args.keypair)?,
			json: args.json,
			simulate_only: args.simulate,
		})
	}

	/// The signer's address.
	pub fn signer(&self) -> Pubkey {
		self.signer.pubkey()
	}

	/// Fetch an account, failing when it does not exist.
	pub fn account(&self, address: &Pubkey) -> Result<FetchedAccount, CliError> {
		self.rpc
			.get_account_with_commitment(address, CommitmentConfig::confirmed())
			.map_err(|error| CliError::Rpc(error.to_string()))?
			.value
			.map(|account| {
				FetchedAccount {
					owner: account.owner,
					data: account.data,
				}
			})
			.ok_or(CliError::AccountNotFound(*address))
	}

	/// Whether an account exists.
	pub fn exists(&self, address: &Pubkey) -> Result<bool, CliError> {
		Ok(self
			.rpc
			.get_account_with_commitment(address, CommitmentConfig::confirmed())
			.map_err(|error| CliError::Rpc(error.to_string()))?
			.value
			.is_some())
	}

	fn transaction(&self, instructions: &[Instruction]) -> Result<Transaction, CliError> {
		let blockhash = self
			.rpc
			.get_latest_blockhash()
			.map_err(|error| CliError::Rpc(error.to_string()))?;
		let message = Message::new(instructions, Some(&self.signer.pubkey()));
		let mut transaction = Transaction::new_unsigned(message);
		transaction
			.try_sign(&[&self.signer], blockhash)
			.map_err(|error| CliError::TransactionFailed(error.to_string()))?;
		Ok(transaction)
	}

	/// Simulate `instructions` and collect the AMM's events.
	pub fn simulate(&self, instructions: &[Instruction]) -> Result<Simulation, CliError> {
		let transaction = self.transaction(instructions)?;
		let result = self
			.rpc
			.simulate_transaction_with_config(
				&transaction,
				RpcSimulateTransactionConfig {
					sig_verify: false,
					replace_recent_blockhash: true,
					commitment: Some(CommitmentConfig::confirmed()),
					..RpcSimulateTransactionConfig::default()
				},
			)
			.map_err(|error| CliError::Rpc(error.to_string()))?
			.value;
		let logs = result.logs.unwrap_or_default();
		if let Some(error) = result.err {
			return Err(CliError::SimulationFailed {
				error: error.to_string(),
				logs: logs.join("\n"),
			});
		}
		Ok(Simulation {
			units: result.units_consumed.unwrap_or_default(),
			events: program_events(&logs, &PINA_AMM_ID),
		})
	}

	/// Send `instructions`, or only simulate them under `--simulate`.
	///
	/// Returns the transaction signature, or `None` for a simulation.
	pub fn send(&self, instructions: &[Instruction]) -> Result<Option<String>, CliError> {
		if self.simulate_only {
			let simulation = self.simulate(instructions)?;
			if !self.json {
				println!(
					"simulation succeeded using {} compute units",
					simulation.units
				);
			}
			return Ok(None);
		}
		let transaction = self.transaction(instructions)?;
		self.rpc
			.send_and_confirm_transaction(&transaction)
			.map(|signature| Some(signature.to_string()))
			.map_err(|error| CliError::TransactionFailed(error.to_string()))
	}
}

/// Map a cluster name or URL to an RPC endpoint.
pub fn resolve_endpoint(value: &str) -> Result<String, CliError> {
	let endpoint = match value {
		"mainnet" | "mainnet-beta" | "m" => MAINNET,
		"devnet" | "d" => DEVNET,
		"testnet" | "t" => TESTNET,
		"localhost" | "localnet" | "l" => LOCALHOST,
		url if url.starts_with("https://") => url,
		url if url.starts_with("http://") => {
			let host = url
				.trim_start_matches("http://")
				.split(['/', ':'])
				.next()
				.unwrap_or_default();
			if !matches!(host, "localhost" | "127.0.0.1" | "[::1]") {
				return Err(CliError::InsecureEndpoint(url.to_owned()));
			}
			url
		}
		other => return Err(CliError::InvalidEndpoint(other.to_owned())),
	};
	Ok(endpoint.to_owned())
}

/// Read a Solana CLI keypair file: a JSON array of 64 bytes.
fn load_keypair(path: &str) -> Result<Keypair, CliError> {
	let expanded = match path.strip_prefix("~/") {
		Some(rest) => {
			let home = std::env::var("HOME").map_err(|_| {
				CliError::Keypair {
					path: path.to_owned(),
					reason: "HOME is not set".to_owned(),
				}
			})?;
			format!("{home}/{rest}")
		}
		None => path.to_owned(),
	};
	let failure = |reason: String| {
		CliError::Keypair {
			path: path.to_owned(),
			reason,
		}
	};
	let contents =
		std::fs::read_to_string(&expanded).map_err(|error| failure(error.to_string()))?;
	let bytes: Vec<u8> =
		serde_json::from_str(&contents).map_err(|error| failure(error.to_string()))?;
	Keypair::try_from(bytes.as_slice()).map_err(|error| failure(error.to_string()))
}

/// The `Program data:` records `program` itself emitted.
///
/// Runtime logs frame every invocation with `Program <id> invoke [n]` and a
/// matching `success` or `failed` line. A data record belongs to whichever
/// program is innermost at that point, so records written by programs the AMM
/// calls (or by programs that call the AMM) are never attributed to it.
pub fn program_events(logs: &[String], program: &Pubkey) -> Vec<Vec<u8>> {
	let program = program.to_string();
	let mut stack: Vec<&str> = Vec::new();
	let mut events = Vec::new();
	for line in logs {
		let Some(rest) = line.strip_prefix("Program ") else {
			continue;
		};
		if let Some(data) = rest.strip_prefix("data: ") {
			if stack.last() == Some(&program.as_str())
				&& let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data.trim())
			{
				events.push(bytes);
			}
			continue;
		}
		let Some((id, tail)) = rest.split_once(' ') else {
			continue;
		};
		if tail.starts_with("invoke [") {
			stack.push(id);
		} else if tail == "success" || tail.starts_with("failed") {
			stack.pop();
		}
	}
	events
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn resolves_cluster_names_and_rejects_remote_http() {
		assert_eq!(resolve_endpoint("devnet").expect("devnet"), DEVNET);
		assert_eq!(resolve_endpoint("l").expect("localhost"), LOCALHOST);
		assert_eq!(
			resolve_endpoint("https://rpc.example.com").expect("https"),
			"https://rpc.example.com"
		);
		assert!(resolve_endpoint("http://127.0.0.1:8899").is_ok());
		assert!(matches!(
			resolve_endpoint("http://rpc.example.com"),
			Err(CliError::InsecureEndpoint(_))
		));
		assert!(matches!(
			resolve_endpoint("ftp://example.com"),
			Err(CliError::InvalidEndpoint(_))
		));
	}

	#[test]
	fn attributes_events_to_the_innermost_program() {
		let amm = Pubkey::new_unique();
		let other = Pubkey::new_unique();
		let record = |bytes: &[u8]| {
			format!(
				"Program data: {}",
				base64::engine::general_purpose::STANDARD.encode(bytes)
			)
		};
		let logs = vec![
			format!("Program {other} invoke [1]"),
			record(&[9]),
			format!("Program {amm} invoke [2]"),
			record(&[1, 2]),
			"Program log: unrelated".to_owned(),
			format!("Program {amm} consumed 10 of 200000 compute units"),
			format!("Program {amm} success"),
			record(&[8]),
			format!("Program {other} success"),
			format!("Program {amm} invoke [1]"),
			record(&[3]),
			format!("Program {amm} success"),
		];
		assert_eq!(program_events(&logs, &amm), vec![vec![1, 2], vec![3]]);
	}
}
