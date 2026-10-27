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

	/// Authorize a transaction only when it will be submitted.
	fn signed_transaction(&self, instructions: &[Instruction]) -> Result<Transaction, CliError> {
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
		// Security: A signed simulation could be broadcast by the RPC. Keep
		// placeholder signatures and blockhash; the RPC replaces the latter.
		let message = Message::new(instructions, Some(&self.signer.pubkey()));
		let transaction = Transaction::new_unsigned(message);
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
		let transaction = self.signed_transaction(instructions)?;
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
			if !is_loopback_authority(&url["http://".len()..]) {
				return Err(CliError::InsecureEndpoint(url.to_owned()));
			}
			url
		}
		other => return Err(CliError::InvalidEndpoint(other.to_owned())),
	};
	Ok(endpoint.to_owned())
}

/// Whether the authority of a URL (everything after the scheme) names a
/// loopback host. Userinfo is rejected outright: in `localhost:1@example.com`
/// the real host is `example.com`.
fn is_loopback_authority(rest: &str) -> bool {
	let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
	if authority.contains('@') {
		return false;
	}
	let host = if let Some(bracketed) = authority.strip_prefix('[') {
		bracketed.split(']').next().unwrap_or_default()
	} else {
		authority.split(':').next().unwrap_or_default()
	};
	matches!(host, "localhost" | "127.0.0.1" | "::1")
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
	use std::sync::Arc;
	use std::sync::Mutex;

	use serde_json::Value;
	use serde_json::json;
	use solana_instruction::AccountMeta;
	use solana_rpc_client::mock_sender::MockSender;
	use solana_rpc_client::rpc_client::RpcClientConfig;
	use solana_rpc_client::rpc_sender::RpcSender;
	use solana_rpc_client::rpc_sender::RpcTransportStats;
	use solana_rpc_client_api::request::RpcRequest;

	use super::*;

	type Requests = Arc<Mutex<Vec<(RpcRequest, Value)>>>;

	/// Capture the real RPC client's serialized requests without a network.
	struct RecordingSender {
		inner: MockSender,
		requests: Requests,
	}

	#[async_trait::async_trait]
	impl RpcSender for RecordingSender {
		async fn send(
			&self,
			request: RpcRequest,
			params: Value,
		) -> solana_rpc_client_api::client_error::Result<Value> {
			self.requests
				.lock()
				.expect("requests")
				.push((request, params.clone()));
			self.inner.send(request, params).await
		}

		fn get_transport_stats(&self) -> RpcTransportStats {
			self.inner.get_transport_stats()
		}

		fn url(&self) -> String {
			self.inner.url()
		}
	}

	/// Use the SDK's RPC responses, including a usable recent blockhash.
	fn recording_context(simulate_only: bool, simulation: Option<Value>) -> (Context, Requests) {
		let requests = Requests::default();
		let mocks = simulation
			.map(|response| (RpcRequest::SimulateTransaction, response))
			.into_iter()
			.collect();
		let context = Context {
			rpc: RpcClient::new_sender(
				RecordingSender {
					inner: MockSender::new_with_mocks("succeeds", mocks),
					requests: Arc::clone(&requests),
				},
				RpcClientConfig::with_commitment(CommitmentConfig::confirmed()),
			),
			signer: Keypair::new(),
			json: true,
			simulate_only,
		};

		(context, requests)
	}

	/// A mutation requiring the user's authority, like quote and send instructions.
	fn signed_instruction(context: &Context) -> Instruction {
		Instruction {
			program_id: PINA_AMM_ID,
			accounts: vec![AccountMeta::new(context.signer(), true)],
			data: vec![1, 2, 3],
		}
	}

	/// Inspect the exact transaction bytes passed to the RPC transport.
	fn decode_transaction(params: &Value) -> Transaction {
		assert_eq!(params[1]["encoding"], "base64");
		let bytes = base64::engine::general_purpose::STANDARD
			.decode(params[0].as_str().expect("encoded transaction"))
			.expect("base64 transaction");

		wincode::deserialize(&bytes).expect("wire transaction")
	}

	/// No RPC should receive authorization to broadcast a simulation.
	fn assert_unsigned_simulation(
		context: &Context,
		instructions: &[Instruction],
		requests: &Requests,
	) {
		let requests = requests.lock().expect("requests");
		let simulations: Vec<_> = requests
			.iter()
			.filter(|(request, _)| *request == RpcRequest::SimulateTransaction)
			.collect();
		assert_eq!(simulations.len(), 1);

		let params = &simulations[0].1;
		let transaction = decode_transaction(params);
		assert!(
			!transaction.is_signed(),
			"simulation exposed broadcast authorization"
		);
		assert_eq!(params[1]["sigVerify"], false);
		assert_eq!(params[1]["replaceRecentBlockhash"], true);
		assert_eq!(
			transaction.message,
			Message::new(instructions, Some(&context.signer()))
		);
		assert_eq!(
			transaction.signatures.len(),
			usize::from(transaction.message.header.num_required_signatures)
		);
		assert!(
			transaction
				.signatures
				.iter()
				.all(|signature| signature.as_ref() == [0; 64])
		);
		assert_ne!(
			transaction.signatures[0],
			context.signer.sign_message(&transaction.message_data())
		);
		assert!(!requests.iter().any(|(request, _)| {
			matches!(
				request,
				RpcRequest::GetLatestBlockhash | RpcRequest::SendTransaction
			)
		}));
	}

	#[test]
	fn quote_and_simulate_only_requests_are_not_broadcastable() {
		for simulate_only in [false, true] {
			let (context, requests) = recording_context(simulate_only, None);
			let instructions = [signed_instruction(&context)];

			if simulate_only {
				assert_eq!(context.send(&instructions).expect("simulate only"), None);
			} else {
				context.simulate(&instructions).expect("quote simulation");
			}

			assert_unsigned_simulation(&context, &instructions, &requests);
		}
	}

	#[test]
	fn failed_simulations_remain_unsigned_and_report_logs() {
		for simulate_only in [false, true] {
			let response = json!({
				"context": { "slot": 1 },
				"value": {
					"err": { "InstructionError": [0, { "Custom": 7 }] },
					"logs": ["Program log: rejected"]
				}
			});
			let (context, requests) = recording_context(simulate_only, Some(response));
			let instructions = [signed_instruction(&context)];
			let error = if simulate_only {
				context.send(&instructions).err()
			} else {
				context.simulate(&instructions).err()
			};

			assert!(
				matches!(error, Some(CliError::SimulationFailed { logs, .. }) if logs == "Program log: rejected")
			);
			assert_unsigned_simulation(&context, &instructions, &requests);
		}
	}

	#[test]
	fn rpc_simulation_failures_never_fall_through_to_send() {
		for simulate_only in [false, true] {
			let (context, requests) = recording_context(simulate_only, Some(Value::Null));
			let instructions = [signed_instruction(&context)];
			let error = if simulate_only {
				context.send(&instructions).err()
			} else {
				context.simulate(&instructions).err()
			};

			assert!(matches!(error, Some(CliError::Rpc(_))));
			assert_unsigned_simulation(&context, &instructions, &requests);
		}
	}

	#[test]
	fn send_signs_the_rpc_blockhash_and_submits_once() {
		let (context, requests) = recording_context(false, None);
		let instructions = [signed_instruction(&context)];
		let signature = context
			.send(&instructions)
			.expect("send")
			.expect("signature");
		let requests = requests.lock().expect("requests");
		let sends: Vec<_> = requests
			.iter()
			.filter(|(request, _)| *request == RpcRequest::SendTransaction)
			.collect();
		assert_eq!(sends.len(), 1);
		assert_eq!(
			requests
				.iter()
				.filter(|(request, _)| *request == RpcRequest::GetLatestBlockhash)
				.count(),
			1
		);
		assert!(
			!requests
				.iter()
				.any(|(request, _)| *request == RpcRequest::SimulateTransaction)
		);

		let transaction = decode_transaction(&sends[0].1);
		let mut expected = Message::new(&instructions, Some(&context.signer()));
		expected.recent_blockhash = solana_rpc_client::mock_sender::PUBKEY
			.parse()
			.expect("blockhash");
		assert_eq!(transaction.message, expected);
		assert_eq!(
			transaction.signatures,
			vec![context.signer.sign_message(&transaction.message_data())]
		);
		assert!(transaction.is_signed());
		assert_eq!(signature, transaction.signatures[0].to_string());
	}

	#[test]
	fn resolves_cluster_names_and_rejects_remote_http() {
		assert_eq!(resolve_endpoint("devnet").expect("devnet"), DEVNET);
		assert_eq!(resolve_endpoint("l").expect("localhost"), LOCALHOST);
		assert_eq!(
			resolve_endpoint("https://rpc.example.com").expect("https"),
			"https://rpc.example.com"
		);
		assert!(resolve_endpoint("http://127.0.0.1:8899").is_ok());
		assert!(resolve_endpoint("http://localhost").is_ok());
		assert!(resolve_endpoint("http://[::1]:8899/").is_ok());
		for remote in [
			"http://localhost:8899@rpc.example.com",
			"http://localhost.example.com",
			"http://127.0.0.1.example.com:8899",
			"http://[::2]:8899",
		] {
			assert!(
				matches!(resolve_endpoint(remote), Err(CliError::InsecureEndpoint(_))),
				"{remote} must be rejected"
			);
		}
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
