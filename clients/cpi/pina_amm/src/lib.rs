//! `no_std` CPI client for the [Pina AMM](https://github.com/pina-rs/amm).
//!
//! Other on-chain programs use this crate to call the AMM: build the generated
//! instruction struct with its accounts and data, then call `.invoke()` or
//! `.invoke_signed(signers)` with the validated AMM program account. Every
//! type is generated from the program's IDL by `pina generate`; regenerate it
//! instead of editing it.
//!
//! The account parsers in this crate check discriminators only. Before reading
//! another program's account through them, call `assert_owner` with the AMM's
//! program id.
//!
//! See the repository's `docs/cpi.md` for a complete example.

#![no_std]

pub mod generated;
pub use generated::*;
