//! # Pina AMM
//!
//! A permissionless constant-product automated market maker for Solana, built
//! with [Pina](https://github.com/pina-rs/pina).
//!
//! The program follows the shape of Raydium's CP-Swap and keeps only the parts
//! every integration needs:
//!
//! - **Fee tiers** ([`AmmConfig`]) are created by the program's upgrade
//!   authority. Each tier fixes a trade fee, the protocol's share of that fee,
//!   and an optional creator fee. A tier can be restricted to one pool-creator
//!   signer so another program (for example a bonding-curve launchpad) can
//!   create pools that cannot be front-run.
//! - **Pools** ([`Pool`]) are created by anyone under a tier. Each pool
//!   snapshots its tier's rates, so a tier update never changes the economics
//!   of an existing pool.
//! - **Swaps** use `x * y = k` with fees rounded in the pool's favour, for both
//!   exact-input and exact-output trades.
//! - **Liquidity** is represented by a legacy SPL Token LP mint owned by the
//!   pool. A small amount of liquidity is locked forever when the pool is
//!   created, so the pool can never be emptied.
//!
//! Reserves are never stored. A pool's reserve is its vault balance minus the
//! protocol and creator fees it has accrued but not yet paid out, so tokens
//! sent directly to a vault become liquidity for every LP.
//!
//! ## Module map
//!
//! | Module | Contents |
//! | --- | --- |
//! | [`state`] | Account layouts, seeds, and PDA helpers |
//! | [`instructions`] | Instruction data layouts and discriminators |
//! | [`processors`] | Account lists and handlers for every instruction |
//! | [`math`] | Pure, checked swap and liquidity arithmetic |
//! | [`events`] | Events emitted to the transaction log |
//! | [`errors`] | Program error codes |

#![no_std]
#![allow(clippy::inline_always)]
// `AccountView` is a copyable handle, but borrowing it keeps account access and
// mutability explicit at helper boundaries.
#![allow(clippy::trivially_copy_pass_by_ref)]

#[cfg(all(
	not(any(target_os = "solana", target_arch = "bpf")),
	not(feature = "bpf-entrypoint"),
	not(test)
))]
extern crate std;

/// On-chain entrypoint that routes every instruction to its processor.
#[cfg(feature = "bpf-entrypoint")]
pub mod entrypoint;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod processors;
pub mod state;

mod token;
mod upgrade_authority;

pub use errors::*;
pub use events::*;
pub use instructions::*;
pub use math::*;
use pina::*;
pub use processors::*;
pub use state::*;

declare_id!("pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV");
