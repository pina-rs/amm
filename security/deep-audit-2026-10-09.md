# Deep security audit — 2026-10-09

Base: `main` @ `0347be2` (audited before the v0.1.0 release; this branch is rebased onto the release commit `e4f729e`, which only bumps versions and adds release notes). Scope: full repository, centered on a line-by-line review of the on-chain program (`programs/pina_amm/src/`, all ~1,600 lines: state, math, every processor, token helpers, upgrade-authority gate), the generated CPI/Rust/TS/Dart clients as consumed, the `pina_amm_cli` crate, CI and release workflows, `devenv` build pipeline, and `docs/`. Method: the solana-audit skill taxonomy (operations / code / economics layers), applied class by class, with a tooling pass (`cargo test -p pina_amm --lib` — 16/16 pass) and a cross-check against the incident history (Wormhole account spoofing, Cashio fake collateral, Cetus overflow, Raydium CPMM patterns, pump.fun launchpad patterns).

Both repositories were reviewed together, because the bonding curve graduates into this AMM and vendors this repository's CPI client; findings that live on the integration boundary are reported in both repositories.

## Summary

| Severity      | Count                |
| ------------- | -------------------- |
| Critical      | 0                    |
| High          | 0                    |
| Medium        | 0                    |
| Low           | 1 (fixed in this PR) |
| Informational | 4                    |

**Verdict:** the program is well-built. Every historical loss pattern was looked for and is either structurally excluded or explicitly documented as an accepted limitation: accounts are typed-loaded (owner + discriminator + version + size), every stored address is compared before use, token programs must own what they move, `[profile.release] overflow-checks = true` and all arithmetic is checked anyway, fee and output rounding favours the pool, reserves are computed from live vault balances minus accrued fees (so LPs can never withdraw the protocol's or creator's money), and Token-2022 extensions that change transfer semantics are rejected at pool creation. The property tests in `math.rs` genuinely pin the invariants (`k` never decreases, round trips never profit, the locked minimum is unreachable).

## Low (fixed in this PR)

### AMM-1 — The default address can hold pool-creator rights, permanently locking creator fees

1. **Location:** `programs/pina_amm/src/processors/pool.rs` (`CreatePoolInstruction.creator` argument) and `processors/fees.rs` (`SetPoolCreator` accepting `Address::default()`).
2. **Mechanism:** `CollectCreatorFees` requires a signer equal to the stored `creator`. The default address cannot sign, so creator fees accrued to it are unclaimable forever — yet `PoolSnapshot::reserves` subtracts them from the vault balance on every withdrawal, so the value stays locked in the vault where no LP can reach it either. A pool created with `creator = Address::default` (or handed to it later) permanently destroys every subsequent creator fee.
3. **Exploit scenario:** self-inflicted only — the creator is the party who chooses the argument or signs the handover — but it is a silent, irreversible footgun, and the bonding curve passes `launch.creator` into this field at graduation, so a bricked creator upstream propagates here.
4. **Severity justification:** no attacker path to another party's funds; permanent loss of the pool creator's own accruals and a slow leak of withdrawable liquidity. Low.
5. **Fix (this PR):** `CreatePool` and `SetPoolCreator` reject the default address with the new append-only error code `DefaultCreator` (15); both rejection paths are exercised in the Surfpool suite.

## Informational

- **AMM-2 — No events for `UpdateConfig` and `SetPoolCreator`.** Every value-moving instruction emits an event, but tier updates and creator handovers are observable only by diffing accounts. Indexers tracking fee-recipient changes must poll account state. Consider events in a future wire-compatible release (events are append-only in the IDL).
- **AMM-3 — Tier rates apply to pools created later, not at tier-selection time.** This is documented for the AMM's own users, but it also binds launchpads built on restricted tiers (see the bonding curve audit): a graduation pool snapshots whatever the tier says at graduation. The 10% combined cap in `FeeRates::validate` bounds the damage; nothing more to fix here.
- **AMM-4 — Over-funded pre-created vault/LP-mint PDAs strand the surplus lamports forever** (`token.rs::create_pda_account` tops up only the shortfall; excess stays). Pre-funding can no longer block creation (good); the excess is the attacker's own money and unrecoverable by design. Accepted.
- **AMM-5 — `Upgrade authority` is the root of trust for tier creation** (`upgrade_authority.rs` reads it from program data — correctly laid out and canonical-address-checked). Documented in `docs/security.md`; deployments must hold it in a multisig. Not a code defect.

## Verified-correct (selected, this pass)

- Swap accounting under both creator-fee modes and both directions: fees round up, outputs round down, `assert_constant_product` runs on net reserves, and the exact-output path's `gross_up` always covers its own fee (property-tested).
- Reserve accounting: `reserves()` = vault balance − accrued protocol − accrued creator fees, checked with saturation to `VaultAccountingMismatch`; the solvency invariant (balance ≥ accrued) holds through every instruction including external donation, and donations become plain LP liquidity.
- First-depositor share inflation: excluded by `MINIMUM_LIQUIDITY` lock plus the `InsufficientInitialLiquidity` floor; withdrawal is bounded away from the locked amount even when LP is burned outside the program.
- Pool-creation front-running by launchpads: restricted tiers pin `pool_creator_authority`, which `UpdateConfig` cannot change — verified that this field is genuinely immutable in the update processor.
- Token-2022 policy: the extension allowlist excludes every extension that alters transfer amounts or authorities (transfer fee, hooks, permanent delegate, pausing, non-transferable, confidential transfer, close authority); freeze-authority mints are accepted as a documented trade-off.
- The CLI's `resolve_endpoint` rejects non-loopback `http://` endpoints including the `localhost:1@example.com` userinfo trick; keypairs are read from disk and never logged.

## What was not reviewed

On-chain bytecode (the program has not been deployed; no verified-build comparison was possible), the deployed-key custody itself, and the Cloudflare worker beyond a read (it only serves `/api/health` and static assets).

## Recommended actions before release

1. Merge the default-creator rejection (this PR).
2. File the event-coverage gap (AMM-2) as a follow-up decision.
3. Re-run this audit's integration-boundary section together with the bonding curve's report after either program changes its wire format.
