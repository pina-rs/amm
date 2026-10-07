//! `pina-amm`: a command-line interface for the Pina AMM.
//!
//! The binary is a thin layer over the generated `pina_amm_client` crate. It
//! derives every PDA, vault, and associated token account, so commands take
//! the handful of values a person actually chooses: a tier index, two mints,
//! and amounts. Trades and liquidity changes are quoted by simulating the
//! instruction and reading the event the program emits, then sent with a
//! slippage limit derived from that quote.
//!
//! Run `pina-amm --help` for the command reference, or read
//! `docs/cli.md` in the repository.

mod accounts;
mod cli;
mod commands;
mod context;
mod error;

use clap::Parser;

use crate::cli::Cli;
use crate::context::Context;

fn main() {
	let cli = Cli::parse();
	let result = Context::new(&cli.global).and_then(|context| commands::run(&context, cli.command));
	if let Err(error) = result {
		eprintln!("error: {error}");
		std::process::exit(1);
	}
}
