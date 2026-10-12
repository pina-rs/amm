---
amm: major
---

# Price the accumulator with stored reserves so donations cannot reprice the past

The time-weighted price accumulator priced the whole interval since `last_update_timestamp` with the reserves read from the vaults _now_. Reserves are vault balances, so a donation straight to a vault followed by `SyncPool` or a swap in the same transaction repriced time that had already passed; with flash-deposited liquidity (deposits and withdrawals are feeless) the attacker recovered nearly all of the donation.

- `Pool` gains `reserve_0_last` and `reserve_1_last`, the reserves at the last observation. `CreatePool`, both swaps, `Deposit`, `Withdraw`, and `SyncPool` now accrue the elapsed interval with those stored reserves first, then store their own post-instruction reserves, exactly like Uniswap V2's `_update`. Deposits and withdrawals now advance the accumulator too.
- The new pure `PriceOracle::observe` holds the logic; property tests prove the accumulator equals the per-second integral of the observed reserves across mixed swaps, deposits, withdrawals, donations, and syncs.
- `docs/math.md` and `docs/security.md` describe the sampling rules and the remaining limits honestly: the average resists single-transaction manipulation, not a price held across seconds.
- `CreateConfig` emits the new `ConfigCreated` event with the tier's index, rates, authority, and pool-creator restriction.
- The deployed binary embeds a `security.txt` pointing at `SECURITY.md` and the private advisory form.
