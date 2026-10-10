//! Account lists and handlers for every instruction.
//!
//! Each `*Accounts` struct is the ordered account list its instruction expects;
//! the generated clients build the same list. Handlers validate every account
//! before reading or moving value, commit state before any outgoing transfer,
//! and emit one event when they succeed.

mod common;
mod config;
mod fees;
mod liquidity;
mod pool;
mod swap;
mod sync;

pub use config::*;
pub use fees::*;
pub use liquidity::*;
pub use pool::*;
pub use swap::*;
pub use sync::*;
