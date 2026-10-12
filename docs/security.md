# Security model

This page records what the program defends against, how, and which test proves it. It is not an audit. The program has not been independently reviewed yet; see [SECURITY.md](../SECURITY.md) for reporting. The deployed binary embeds a [`security.txt`](https://github.com/neodyme-labs/solana-security-txt) with the same contact and policy links, so explorers and `query-security-txt` can find them from the program address.

## Trust assumptions

| Party             | Can                                                               | Cannot                                          |
| ----------------- | ----------------------------------------------------------------- | ----------------------------------------------- |
| Upgrade authority | Create fee tiers; upgrade the program                             | Touch funds without shipping new code           |
| Tier authority    | Change a tier's rates for **future** pools; collect protocol fees | Change an existing pool's rates; reach LP funds |
| Pool creator      | Collect creator fees; hand them over                              | Change rates or reach LP funds                  |
| Anyone            | Create pools in open tiers, trade, provide liquidity              | Create pools in restricted tiers                |

The upgrade authority is the single point of trust, as for any upgradeable Solana program. Deployments should hold it in a multisig and announce upgrades.

## Invariants

| Invariant                                                                  | Enforcement                                                                | Test                                                                                                                          |
| -------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `x * y` never decreases across a swap                                      | Rounding plus an explicit check                                            | `exact_in_preserves_the_invariant`, `exact_out_preserves_the_invariant` (property tests)                                      |
| Fees, outputs, and inputs round in the pool's favour                       | Math module                                                                | `rounds_fees_up_and_shares_down`, `exact_out_never_undercuts_exact_in`                                                        |
| A deposit and withdrawal round trip never profits                          | Ceiling on deposit, floor on withdraw                                      | `liquidity_round_trip_never_profits`                                                                                          |
| A pool can never be emptied                                                | 1,000 LP locked at creation; withdrawals cannot reach it                   | `liquidity_moves_in_proportion_and_the_minimum_stays_locked`                                                                  |
| Accrued fees never price trades                                            | Reserves exclude them                                                      | `swaps_match_the_documented_formulas_and_accrue_fees`                                                                         |
| Fee collection never touches liquidity                                     | Collection pays at most the accrued amount                                 | `fees_are_collected_only_by_their_owners`                                                                                     |
| Existing pools keep their rates                                            | Rates snapshotted at creation                                              | `tier_updates_only_affect_future_pools`                                                                                       |
| The accumulator prices each second with the reserves that stood through it | Every observation accrues with the stored reserves before storing new ones | `accrual_prices_each_second_with_the_reserves_standing_then`, `the_closed_interval_ignores_the_new_reserves` (property tests) |

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

| Attack                                                                       | Outcome                                                                                                              | Test                                                                                                     |
| ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Pre-fund a vault or LP-mint address to block pool creation                   | Creation tops up, allocates, and assigns instead of failing                                                          | `prefunded_pool_addresses_cannot_block_creation`                                                         |
| Create a launchpad's pool before it does                                     | Restricted tiers require their pool-creator authority                                                                | `restricted_tiers_only_accept_their_pool_creator`                                                        |
| Pass another pool's vault, or a user account as a vault                      | `PoolAccountMismatch`                                                                                                | `swaps_enforce_slippage_limits_and_pool_vaults`                                                          |
| Collect protocol fees through the wrong tier                                 | `PoolAccountMismatch`                                                                                                | `fees_are_collected_only_by_their_owners`                                                                |
| Use a transfer-fee or hook mint to make transfers move less than accounted   | `UnsupportedMint` at creation                                                                                        | `create_pool_rejects_unordered_mints_and_unsupported_extensions`                                         |
| Inflate share price as the first depositor                                   | Locked minimum liquidity                                                                                             | `liquidity_locks_the_minimum` (unit)                                                                     |
| Donate to manipulate LP accounting                                           | Donations simply accrue to all LPs                                                                                   | `donations_accrue_to_liquidity_providers`                                                                |
| Create a tier without authority                                              | `Unauthorized`                                                                                                       | `create_config_requires_the_upgrade_authority`                                                           |
| Name the default address as a pool creator, making its fees unclaimable      | `DefaultCreator`                                                                                                     | `create_pool_rejects_unordered_mints_and_unsupported_extensions`                                         |
| Read the spot price as an oracle                                             | Use the time-weighted accumulator; see [Time-weighted price](#time-weighted-price)                                   | `sync_accumulates_the_time_weighted_price`                                                               |
| Donate to a vault, then `SyncPool` or trade, to reprice the elapsed interval | The elapsed interval is priced with the reserves stored at the last observation; the donation prices only later time | `donations_cannot_reprice_the_elapsed_interval`, `a_donation_cannot_reprice_the_elapsed_interval` (unit) |

## Time-weighted price

The accumulator follows Uniswap V2: each observation prices the interval since the previous one with the reserves recorded _at_ the previous one, then records the reserves the current instruction leaves behind. [math.md](math.md#time-weighted-average-price) has the formula.

What this resists:

- **Single-transaction manipulation.** Anything an attacker does within one transaction, including flash-deposited liquidity, a donation, `SyncPool`, and a withdrawal, changes only the price recorded for the interval that starts at that transaction. If the attacker restores the price before the transaction ends, nothing is recorded at all.
- **Repricing the past.** No instruction can change what has already accrued. A donation counts from the next observation onward, like any other price move.

What it does not resist, and what integrators must account for:

- **Multi-block manipulation.** An attacker who leaves the pool at a false price across a second boundary has that price recorded for as long as it stands. Holding it for `n` seconds moves an `N`-second average by roughly `n / N` of the distortion, and costs whatever arbitrageurs extract while it stands. A validator producing consecutive slots, or an attacker with a censorship path, can hold a price for a few seconds at low cost. Choose a window much longer than the time anyone could hold a distorted price, and weigh that against the value an integration secures.
- **Thin liquidity.** The cost of holding a distorted price scales with pool depth. A shallow pool's average is cheap to move over any window.
- **Stale samples.** A quiet pool's last observation keeps pricing every second until the next one. Sample through `SyncPool` and reject windows whose endpoints are older than you can tolerate.
- **Clock behaviour.** The accumulator uses the cluster's stake-weighted `unix_timestamp`, which moves in whole seconds and can drift from wall-clock time. Time only accrues when the clock moves forward.
- **Pair price only.** The accumulator is a time-weighted arithmetic mean of one pool's price. It is not a cross-venue oracle: it says nothing about other markets and cannot detect a pair whose only liquidity is this pool.

Treat the average as one manipulation-resistant input, not as an oracle that is safe on its own. Lending or liquidation logic should combine it with an independent source, bound how far it may move, or both.

## Static analysis

`devenv shell lint:all` runs Pina's official security lint set, which rejects unchecked asset arithmetic, missing signer or owner checks, and inconsistent token-program usage, alongside `clippy -D warnings`.

## Known limitations

- **Freeze authorities.** A mint with a freeze authority (USDC, for example) can have its pool vault frozen by that authority, halting the pool. This is inherent to such mints; the AMM accepts them because rejecting them would exclude most stablecoins.
- **Spot price is not an oracle.** The pool price is manipulable within a transaction like every constant-product AMM. The time-weighted accumulator resists single-transaction manipulation but not a price held across seconds; see [Time-weighted price](#time-weighted-price) for its limits.
- **Single-step authority transfer.** `UpdateConfig` hands a tier's authority to a new address in one step, so a mistyped address loses control of the tier's rates and protocol fees. A two-step propose-and-accept transfer is planned.
- **No independent audit yet.** Do not deposit material value until one is published.
