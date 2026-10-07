# Security model

This page records what the program defends against, how, and which test proves it. It is not an audit. The program has not been independently reviewed yet; see [SECURITY.md](../SECURITY.md) for reporting.

## Trust assumptions

| Party             | Can                                                               | Cannot                                          |
| ----------------- | ----------------------------------------------------------------- | ----------------------------------------------- |
| Upgrade authority | Create fee tiers; upgrade the program                             | Touch funds without shipping new code           |
| Tier authority    | Change a tier's rates for **future** pools; collect protocol fees | Change an existing pool's rates; reach LP funds |
| Pool creator      | Collect creator fees; hand them over                              | Change rates or reach LP funds                  |
| Anyone            | Create pools in open tiers, trade, provide liquidity              | Create pools in restricted tiers                |

The upgrade authority is the single point of trust, as for any upgradeable Solana program. Deployments should hold it in a multisig and announce upgrades.

## Invariants

| Invariant                                            | Enforcement                                              | Test                                                                                     |
| ---------------------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `x * y` never decreases across a swap                | Rounding plus an explicit check                          | `exact_in_preserves_the_invariant`, `exact_out_preserves_the_invariant` (property tests) |
| Fees, outputs, and inputs round in the pool's favour | Math module                                              | `rounds_fees_up_and_shares_down`, `exact_out_never_undercuts_exact_in`                   |
| A deposit and withdrawal round trip never profits    | Ceiling on deposit, floor on withdraw                    | `liquidity_round_trip_never_profits`                                                     |
| A pool can never be emptied                          | 1,000 LP locked at creation; withdrawals cannot reach it | `liquidity_moves_in_proportion_and_the_minimum_stays_locked`                             |
| Accrued fees never price trades                      | Reserves exclude them                                    | `swaps_match_the_documented_formulas_and_accrue_fees`                                    |
| Fee collection never touches liquidity               | Collection pays at most the accrued amount               | `fees_are_collected_only_by_their_owners`                                                |
| Existing pools keep their rates                      | Rates snapshotted at creation                            | `tier_updates_only_affect_future_pools`                                                  |

## Account validation

| Account                           | Check                                                                                                                                  |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `AmmConfig`, `Pool`               | Owned by the program, correct discriminator, schema version, and size (typed load). Pools are only ever created at their canonical PDA |
| Vaults, LP mint                   | Compared with the addresses stored on the pool                                                                                         |
| Fee tier in `CollectProtocolFees` | Must be the pool's stored tier                                                                                                         |
| Token programs                    | Must be SPL Token or Token-2022, and must own the vault they move                                                                      |
| Mints in `CreatePool`             | Canonical order, supported program, allowed extensions only                                                                            |
| Signers                           | Upgrade authority read from program data; tier authority and pool creator compared with stored state                                   |
| Writable accounts                 | Declared `&mut`; Pina rejects a writable account that appears twice                                                                    |

## Attacks considered

| Attack                                                                     | Outcome                                                     | Test                                                             |
| -------------------------------------------------------------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------- |
| Pre-fund a vault or LP-mint address to block pool creation                 | Creation tops up, allocates, and assigns instead of failing | `prefunded_pool_addresses_cannot_block_creation`                 |
| Create a launchpad's pool before it does                                   | Restricted tiers require their pool-creator authority       | `restricted_tiers_only_accept_their_pool_creator`                |
| Pass another pool's vault, or a user account as a vault                    | `PoolAccountMismatch`                                       | `swaps_enforce_slippage_limits_and_pool_vaults`                  |
| Collect protocol fees through the wrong tier                               | `PoolAccountMismatch`                                       | `fees_are_collected_only_by_their_owners`                        |
| Use a transfer-fee or hook mint to make transfers move less than accounted | `UnsupportedMint` at creation                               | `create_pool_rejects_unordered_mints_and_unsupported_extensions` |
| Inflate share price as the first depositor                                 | Locked minimum liquidity                                    | `liquidity_locks_the_minimum` (unit)                             |
| Donate to manipulate LP accounting                                         | Donations simply accrue to all LPs                          | `donations_accrue_to_liquidity_providers`                        |
| Create a tier without authority                                            | `Unauthorized`                                              | `create_config_requires_the_upgrade_authority`                   |

## Static analysis

`devenv shell lint:all` runs Pina's official security lint set, which rejects unchecked asset arithmetic, missing signer or owner checks, and inconsistent token-program usage, alongside `clippy -D warnings`.

## Known limitations

- **Freeze authorities.** A mint with a freeze authority (USDC, for example) can have its pool vault frozen by that authority, halting the pool. This is inherent to such mints; the AMM accepts them because rejecting them would exclude most stablecoins.
- **No oracle.** The pool price is manipulable within a transaction like every constant-product AMM. Do not use the spot price as an oracle.
- **No independent audit yet.** Do not deposit material value until one is published.
