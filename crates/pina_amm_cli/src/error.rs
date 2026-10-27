//! Errors every command can report.

use solana_pubkey::Pubkey;
use thiserror::Error;

/// A failure the CLI reports to the user before exiting with status 1.
#[derive(Debug, Error)]
pub enum CliError {
	/// The `--url` value is neither a cluster name nor an HTTP(S) URL.
	#[error(
		"`{0}` is not a valid RPC endpoint; expected mainnet, devnet, testnet, localhost, or an \
		 https URL"
	)]
	InvalidEndpoint(String),
	/// A remote endpoint was given as plaintext HTTP.
	#[error("plaintext http is only allowed for localhost; use https for `{0}`")]
	InsecureEndpoint(String),
	/// The keypair file could not be read or parsed.
	#[error("could not load keypair `{path}`: {reason}")]
	Keypair {
		/// The path that was read.
		path: String,
		/// Why it failed.
		reason: String,
	},
	/// An RPC request failed.
	#[error("RPC request failed: {0}")]
	Rpc(String),
	/// An account the command needs does not exist.
	#[error("account {0} was not found")]
	AccountNotFound(Pubkey),
	/// An account is not owned by the program the command expected.
	#[error("account {address} is owned by {owner}, not {expected}")]
	WrongOwner {
		/// The account that was fetched.
		address: Pubkey,
		/// Its owner.
		owner: Pubkey,
		/// The owner the command required.
		expected: Pubkey,
	},
	/// Account data did not decode as the expected type.
	#[error("account {0} does not hold the expected Pina AMM data")]
	InvalidAccountData(Pubkey),
	/// A mint is not one of the pool's two mints.
	#[error("mint {mint} is not part of pool {pool}")]
	MintNotInPool {
		/// The mint the user named.
		mint: Pubkey,
		/// The pool it was checked against.
		pool: Pubkey,
	},
	/// Two mints that must differ are the same.
	#[error("a pool needs two different mints")]
	SameMint,
	/// The slippage tolerance is above 100%.
	#[error("slippage of {0} basis points is above 10,000 (100%)")]
	InvalidSlippage(u16),
	/// Simulation failed, so no transaction was sent.
	#[error("simulation failed: {error}\n{logs}")]
	SimulationFailed {
		/// The transaction error.
		error: String,
		/// The program logs, one per line.
		logs: String,
	},
	/// The simulated instruction did not emit the event the command reads.
	#[error("the simulation did not emit the expected Pina AMM event")]
	MissingEvent,
	/// The transaction was sent but failed.
	#[error("transaction failed: {0}")]
	TransactionFailed(String),
	/// Instruction data rejected the provided values.
	#[error("instruction data rejected the provided values")]
	InvalidInstructionData,
}

impl CliError {
	/// Render a command failure for stderr, escaping terminal controls.
	pub fn diagnostic(&self, json: bool) -> String {
		let message = self.to_string();
		let mut safe = String::with_capacity(message.len());

		// RPC messages and simulation logs can contain CSI, OSC, and C1 controls.
		for character in message.chars() {
			if character.is_control() && !matches!(character, '\n' | '\t') {
				safe.extend(character.escape_debug());
			} else {
				safe.push(character);
			}
		}

		if json {
			serde_json::json!({ "error": safe }).to_string()
		} else {
			format!("error: {safe}")
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn readable_diagnostics_preserve_unicode_and_log_layout() {
		let error = CliError::SimulationFailed {
			error: "transaction rejected".to_owned(),
			logs: "Program log: café\n\tvalue=42".to_owned(),
		};

		assert_eq!(
			error.diagnostic(false),
			"error: simulation failed: transaction rejected\nProgram log: café\n\tvalue=42"
		);
	}

	#[test]
	fn remote_errors_escape_all_c0_and_c1_controls_except_newlines_and_tabs() {
		let controls: String = (0..=0x9f)
			.filter_map(char::from_u32)
			.filter(|character| character.is_control())
			.collect();
		let errors = [
			CliError::Rpc(controls.clone()),
			CliError::TransactionFailed(controls.clone()),
			CliError::SimulationFailed {
				error: controls.clone(),
				logs: controls,
			},
		];

		for error in errors {
			for json in [false, true] {
				let diagnostic = error.diagnostic(json);

				assert!(diagnostic.chars().all(|character| {
					!character.is_control() || matches!(character, '\n' | '\t')
				}));

				if json {
					let value: serde_json::Value =
						serde_json::from_str(&diagnostic).expect("JSON diagnostic");

					assert!(
						value["error"]
							.as_str()
							.expect("error string")
							.contains("\\u{1b}")
					);
				} else {
					assert!(diagnostic.contains("\\u{1b}"));
				}
			}
		}
	}
}
