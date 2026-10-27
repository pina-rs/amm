//! A seeded, deterministic fuzzer over the program's global invariants.
//!
//! Random operation sequences run against one real pool on the Surfnet, and
//! after every transaction the suite asserts the invariants
//! `docs/security.md` promises: the vaults stay solvent against the fees
//! accrued in them, the locked LP minimum stays locked, the price
//! accumulator and its clock only move forward, the reserves never empty,
//! and no swap ever decreases the constant product. The generator is a
//! fixed-seed xorshift, so a failure reproduces from the seed alone.

use pina_amm_client::instructions::CollectCreatorFees;
use pina_amm_client::instructions::CollectCreatorFeesInstructionData;
use pina_amm_client::instructions::CollectProtocolFees;
use pina_amm_client::instructions::CollectProtocolFeesInstructionData;
use pina_test::Signer;

use crate::PoolFixture;
use crate::deposit_instruction;
use crate::funded_trader;
use crate::harness::Harness;
use crate::install_admin;
use crate::liquidity_accounts;
use crate::pool_state;
use crate::reserves;
use crate::spl_pool;
use crate::swap_exact_in_instruction;
use crate::swap_exact_out_instruction;
use crate::sync_instruction;
use crate::withdraw_instruction;

/// xorshift64*: three shifts and a multiply, no allocation.
struct Rng(u64);

impl Rng {
	fn next_u64(&mut self) -> u64 {
		let mut x = self.0;
		x ^= x >> 12;
		x ^= x << 25;
		x ^= x >> 27;
		self.0 = x;
		x.wrapping_mul(0x2545_F491_4F6C_DD1D)
	}

	/// A value in `1..=bound`.
	fn up_to(&mut self, bound: u64) -> u64 {
		1 + self.next_u64() % bound
	}

	fn boolean(&mut self) -> bool {
		self.next_u64() & 1 == 1
	}
}

/// The pool facts each invariant is asserted against from the previous step.
#[derive(Default)]
struct InvariantState {
	price_0_cumulative: [u128; 2],
	last_update_timestamp: u64,
}

fn constant_product(h: &Harness, fixture: &PoolFixture) -> u128 {
	let (reserve_0, reserve_1) = reserves(h, fixture);
	u128::from(reserve_0) * u128::from(reserve_1)
}

fn assert_invariants(h: &Harness, fixture: &PoolFixture, previous: &mut InvariantState) {
	let pool = pool_state(h, &fixture.pool);
	assert!(
		pool.lp_supply >= crate::MINIMUM_LIQUIDITY,
		"the locked minimum must stay locked, saw {}",
		pool.lp_supply
	);
	let (balance_0, balance_1) = (
		h.token_balance(&fixture.vault_0),
		h.token_balance(&fixture.vault_1),
	);
	assert!(
		balance_0 >= pool.protocol_fees0 + pool.creator_fees0,
		"vault 0 holds {} against {} of accrued fees",
		balance_0,
		pool.protocol_fees0 + pool.creator_fees0
	);
	assert!(
		balance_1 >= pool.protocol_fees1 + pool.creator_fees1,
		"vault 1 holds {} against {} of accrued fees",
		balance_1,
		pool.protocol_fees1 + pool.creator_fees1
	);
	// Numeric order compares the high limb first.
	let current = crate::cumulative(&pool);
	assert!(
		current
			.iter()
			.rev()
			.cmp(previous.price_0_cumulative.iter().rev())
			.is_ge()
	);
	assert!(pool.last_update_timestamp >= previous.last_update_timestamp);
	previous.price_0_cumulative = current;
	previous.last_update_timestamp = pool.last_update_timestamp;
	let (reserve_0, reserve_1) = reserves(h, fixture);
	assert!(
		reserve_0 > 0 && reserve_1 > 0,
		"a pool can never be emptied, saw ({reserve_0}, {reserve_1})"
	);
}

#[test]
#[ignore = "run with `pina test`"]
fn random_operation_sequences_preserve_the_invariants() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let admin = install_admin(&h);
		let fixture = spl_pool(&h, &admin, 0);
		let trader = funded_trader(&h, &fixture, 2_000_000_000);
		let mut rng = Rng(0x7069_6E61_6D6D);
		let mut previous = InvariantState {
			price_0_cumulative: [0; 2],
			last_update_timestamp: 0,
		};
		assert_invariants(&h, &fixture, &mut previous);

		for _ in 0..60 {
			match rng.up_to(8) {
				1..=3 => {
					// Sell up to 1% of the smaller initial reserve.
					let k_before = constant_product(&h, &fixture);
					let direction = rng.boolean();
					h.send(
						&[swap_exact_in_instruction(
							&fixture,
							&trader.pubkey(),
							direction,
							rng.up_to(10_000_000),
							0,
						)],
						&[&trader],
					)
					.expect("swap exact in");
					assert!(
						constant_product(&h, &fixture) >= k_before,
						"a swap must never decrease the constant product"
					);
				}
				4 => {
					// Buy up to 0.1% of the smaller initial reserve.
					let k_before = constant_product(&h, &fixture);
					let direction = rng.boolean();
					h.send(
						&[swap_exact_out_instruction(
							&fixture,
							&trader.pubkey(),
							direction,
							rng.up_to(1_000_000),
							u64::MAX,
						)],
						&[&trader],
					)
					.expect("swap exact out");
					assert!(
						constant_product(&h, &fixture) >= k_before,
						"a swap must never decrease the constant product"
					);
				}
				5 => {
					h.send(
						&[deposit_instruction(
							&fixture,
							&fixture.owner.pubkey(),
							rng.up_to(100_000),
							(u64::MAX, u64::MAX),
						)],
						&[&fixture.owner],
					)
					.expect("deposit");
				}
				6 => {
					let (_, _, lp_account) = liquidity_accounts(&fixture, &fixture.owner.pubkey());
					let held = h.token_balance(&lp_account);
					// Withdraw at most a small slice, so the reserves stay
					// deep enough for every other operation to succeed: a
					// near-total withdrawal is a legitimate way to shrink the
					// pool to its locked minimum, and the exact-output swaps
					// below would then correctly start failing.
					let bounded = held.min(100_000);
					if held > 0 {
						h.send(
							&[withdraw_instruction(
								&fixture,
								&fixture.owner.pubkey(),
								rng.up_to(bounded),
								(0, 0),
							)],
							&[&fixture.owner],
						)
						.expect("withdraw");
					}
				}
				7 => h.send(&[sync_instruction(&fixture)], &[]).expect("sync"),
				_ => {
					// Both collectors sweep everything into the owner's
					// token accounts, which always exist.
					let (recipient_0, recipient_1, _) =
						liquidity_accounts(&fixture, &fixture.owner.pubkey());
					h.send(
						&[CollectProtocolFees::new(
							admin.pubkey(),
							fixture.config,
							fixture.pool,
							fixture.vault_0,
							fixture.vault_1,
							recipient_0,
							recipient_1,
							fixture.program_0,
							fixture.program_1,
						)
						.instruction(
							CollectProtocolFeesInstructionData::new(|data| {
								data.maximum_amount0.set(u64::MAX);
								data.maximum_amount1.set(u64::MAX);
							})
							.expect("collect protocol data"),
						)],
						&[&admin],
					)
					.expect("collect protocol fees");
					h.send(
						&[CollectCreatorFees::new(
							fixture.owner.pubkey(),
							fixture.pool,
							fixture.vault_0,
							fixture.vault_1,
							recipient_0,
							recipient_1,
							fixture.program_0,
							fixture.program_1,
						)
						.instruction(
							CollectCreatorFeesInstructionData::new(|data| {
								data.maximum_amount0.set(u64::MAX);
								data.maximum_amount1.set(u64::MAX);
							})
							.expect("collect creator data"),
						)],
						&[&fixture.owner],
					)
					.expect("collect creator fees");
					let pool = pool_state(&h, &fixture.pool);
					assert_eq!(
						(
							pool.protocol_fees0,
							pool.protocol_fees1,
							pool.creator_fees0,
							pool.creator_fees1
						),
						(0, 0, 0, 0),
						"a full sweep leaves nothing accrued"
					);
				}
			}
			assert_invariants(&h, &fixture, &mut previous);
		}

		h.stop().expect("stop");
	});
}
