//! SBF entrypoint.
//!
//! [`dispatch_entrypoint!`] reads the instruction discriminator before any
//! account, routes to the matching `*Accounts` struct, and serves the reserved
//! `Migrate` instruction for every account type listed on
//! [`crate::AmmInstruction`].

use pina::*;

use crate::*;

dispatch_entrypoint!(AmmInstruction);
