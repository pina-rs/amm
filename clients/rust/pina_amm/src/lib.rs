//! Rust client for the [Pina AMM](https://github.com/pina-rs/amm).
//!
//! Every type in this crate is generated from the program's IDL by
//! `pina generate`; regenerate it instead of editing it. The crate gives
//! off-chain Rust code:
//!
//! - instruction builders that take each instruction's ordered accounts and
//!   data (`instructions::SwapExactIn`, `instructions::CreatePool`, ...);
//! - account decoders for `AmmConfig` and `Pool` (`accounts`);
//! - event decoders for every log record the program emits (`events`);
//! - the program's error codes (`errors`) and address (`programs`).
//!
//! Decoders check discriminators and schema versions only. Before trusting a
//! decoded account, compare the account's owner with [`PINA_AMM_ID`] or fetch
//! it from a derived PDA address.
//!
//! See the repository's `docs/rust-client.md` for complete examples.

pub mod generated;
pub use generated::programs::*;
pub use generated::*;
