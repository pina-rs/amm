# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/pina-rs/amm/releases/tag/v0.2.0) (2026-10-28)

Grouped release for `amm`.

### Breaking changes

#### Require explicit absolute CLI limits

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #34](https://github.com/pina-rs/amm/pull/34)

Replace simulation-derived slippage bounds with required caller-provided minimum output or maximum input for swaps and two explicit token limits for deposits and withdrawals. Remove `--slippage-bps`; keep swap quotes advisory and reject events describing another swap. Command results report submitted limits rather than simulated execution amounts.

#### Preserve the full time-weighted price integral

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #33](https://github.com/pina-rs/amm/pull/33)

The canonical price accumulator in `Pool` and `PoolSynced` now combines `price_0_cumulative_last` (low) with `price_0_cumulative_high_last` (high) into a 256-bit value. All generated clients expose both limbs. Integrators must reconstruct the full value before subtracting samples; every supported reserve ratio and timestamp window has a delta below `2^192`, so the complete integral is recoverable without halting trading. Take both samples after migration, which initializes the high limb to zero and cannot recover previously lost cycles.

#### Deploy under the mainnet program identity

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #19](https://github.com/pina-rs/amm/pull/19)

The program's address moves from `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV` to `pAMMzVPRwtrxQs2CjLTUUpfCAp9XdvVmwwVhpAuCobi` before its first mainnet deployment.

- `declare_id!` was synced with `pina keys sync`, and `pina migrations create` rebound the unpublished migration history (no publication receipts existed) to the new ID.
- The TypeScript, Dart, Rust, and CPI clients were regenerated: `PINA_AMM_PROGRAM_ADDRESS`, `PINA_AMM_ID`, the event source address, and every PDA helper's default program address now name the new ID, so every config, pool, LP mint, and vault PDA changes with it.
- Integrators that hard-code the old ID, or CPI callers that pin an older `pina_amm_cpi`, must update before calling the deployed program.

#### Reject the default address as a pool creator

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #9](https://github.com/pina-rs/amm/pull/9)

`CreatePool` and `SetPoolCreator` reject the default address with the new `DefaultCreator` error. Creator fees accrued to the default address could never be collected, yet they still reduce the reserves every withdrawal pays from.

#### Time-weighted price accumulator and admin events

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #10](https://github.com/pina-rs/amm/pull/10)

- Every pool now sums its price over time: `price_0_cumulative_last` (Q64.64 price of token 1 in token 0, times seconds) advances on every swap with the pre-trade price and on the new permissionless `SyncPool` instruction. Sample it twice and divide by the elapsed time for a manipulation-resistant average, exactly like Uniswap V2; see `docs/math.md`.
- `UpdateConfig` emits `ConfigUpdated` and `SetPoolCreator` emits `PoolCreatorChanged`, so indexers can follow tier-rate and fee-recipient changes without polling accounts.
- `SyncPool` emits `PoolSynced` with the reserves and the accumulator after the advance.
- `Pool` grows two fields; `pina migrations create` recorded the automatic transition and the generated clients, IDL, and ABI layout test are regenerated.

#### Wrap the price accumulator instead of halting the pool

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #16](https://github.com/pina-rs/amm/pull/16)

`advance_price_accumulator` now wraps modulo `2^128`, like Uniswap V2, instead of failing with `MathOverflow`. The accumulator only ever grows, so a checked sum would have failed every swap and `SyncPool` permanently once a pool with an extreme price ratio had accumulated enough time, with no way to recover short of a program upgrade. Integrators must subtract samples with wrapping arithmetic; `docs/math.md` shows how. The function no longer returns a `Result`.

#### Price the accumulator with stored reserves so donations cannot reprice the past

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #18](https://github.com/pina-rs/amm/pull/18)

The time-weighted price accumulator priced the whole interval since `last_update_timestamp` with the reserves read from the vaults _now_. Reserves are vault balances, so a donation straight to a vault followed by `SyncPool` or a swap in the same transaction repriced time that had already passed; with flash-deposited liquidity (deposits and withdrawals are feeless) the attacker recovered nearly all of the donation.

- `Pool` gains `reserve_0_last` and `reserve_1_last`, the reserves at the last observation. `CreatePool`, both swaps, `Deposit`, `Withdraw`, and `SyncPool` now accrue the elapsed interval with those stored reserves first, then store their own post-instruction reserves, exactly like Uniswap V2's `_update`. Deposits and withdrawals now advance the accumulator too.
- The new pure `PriceOracle::observe` holds the logic; property tests prove the accumulator equals the per-second integral of the observed reserves across mixed swaps, deposits, withdrawals, donations, and syncs.
- `docs/math.md` and `docs/security.md` describe the sampling rules and the remaining limits honestly: the average resists single-transaction manipulation, not a price held across seconds.
- `CreateConfig` emits the new `ConfigCreated` event with the tier's index, rates, authority, and pool-creator restriction.
- The deployed binary embeds a `security.txt` pointing at `SECURITY.md` and the private advisory form.

### Features

#### Client journeys and a performance gate

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #11](https://github.com/pina-rs/amm/pull/11)

- Every instruction now has one ordered journey on a live Surfnet through the generated Rust client (`every_instruction_runs_in_one_journey`) and through the generated TypeScript client (`typescript_client_runs_every_instruction`), which the Rust side drives with funded fixtures and then checks the resulting on-chain state. The TypeScript journey asserts the client's PDA helpers reproduce the Rust-derived addresses before sending anything.
- The journey's keypairs are seeded, so identical binaries measure identical compute units.
- A `performance` workflow benchmarks the pull request against its base: deployed binary size plus every instruction's compute units (median of three journey runs), posted as one consolidated PR comment. The policy fails the check on any binary-size increase or a compute-unit regression above 2%, unless the pull request carries `performance-approved`. `scripts/benchmark.ts` and `scripts/compare-benchmarks.ts` run the same pipeline locally.
- `docs/performance.md` documents the pipeline and the policy.

### Fixes

#### Test-only hardening: exact-rounding proofs, mutation gaps, invariant fuzzing

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #29](https://github.com/pina-rs/amm/pull/29)

No program behavior changes; the `patch` bump is what the changeset API classification asks for a test-module edit inside the program package.

The math module's tests now pin the exact rounding of every quoted amount by cross-multiplication — fees as exact ceilings, outputs as exact floors, shares as exact floors — closing the coverage gaps a first cargo-mutants run surfaced: exact-output fee amounts were never checked numerically, the constant-product guard's rejection path was never exercised, and the zero-sided liquidity amounts were untested. A seeded Surfpool fuzzer additionally drives one pool through random swap, deposit, withdraw, sync, and collection sequences, asserting the security model's global invariants after every transaction, and an extreme-ratio pool now proves end to end that trading continues after the accumulator wraps.

- **Escape terminal controls in CLI diagnostics.** Neutralize control characters in every command error at the stderr rendering boundary, including failed simulation logs and RPC errors. Under `--json`, command errors emit one JSON object on stderr with an `error` string.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #35](https://github.com/pina-rs/amm/pull/35)
- **Trim the deployed program and every instruction's compute.** Compiles the pool PDA's signer-seed builder once instead of a copy per call site: the deployed binary shrinks by 520 bytes and swaps, deposits, and withdrawals each get slightly cheaper. No account, instruction, event, or wire format changes.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #15](https://github.com/pina-rs/amm/pull/15)
- **Keep simulation transactions unsigned.** Construct quote and `--simulate` transactions with placeholder signatures and a default blockhash. Fetch a recent blockhash and sign only when sending a transaction, so an RPC cannot broadcast a captured simulation with the user's authority.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #32](https://github.com/pina-rs/amm/pull/32)

## [0.1.0](https://github.com/pina-rs/amm/releases/tag/v0.1.0) (2026-10-08)

Grouped release for `amm`.

### Features

#### Launch the Pina AMM program, clients, and CLI

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #1](https://github.com/pina-rs/amm/pull/1)

The first release of the Pina AMM: a constant-product market maker built with Pina 0.23.

- **Program** (`pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV`): fee tiers gated by the upgrade authority, permissionless pool creation with optional pool-creator restriction, exact-input and exact-output swaps, proportional liquidity, protocol and creator fee accrual and collection, and Token-2022 support for transfer-neutral extensions. Swaps cost about 4,700 compute units.
- **`pina_amm_client`**, **`pina_amm_cpi`**, **`@pina-rs/amm`**, and **`pina_amm`** (Dart): generated from the program IDL with the version-0 migration baseline.
- **`pina_amm_cli`**: the `pina-amm` binary, which quotes by simulation and applies slippage limits.

An end-to-end Surfpool suite drives every instruction, the generated Rust client, and the CLI against the compiled program, and enforces compute-unit ceilings.
