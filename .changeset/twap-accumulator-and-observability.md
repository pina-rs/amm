---
amm: major
---

# Time-weighted price accumulator and admin events

- Every pool now sums its price over time: `price_0_cumulative_last` (Q64.64 price of token 1 in token 0, times seconds) advances on every swap with the pre-trade price and on the new permissionless `SyncPool` instruction. Sample it twice and divide by the elapsed time for a manipulation-resistant average, exactly like Uniswap V2; see `docs/math.md`.
- `UpdateConfig` emits `ConfigUpdated` and `SetPoolCreator` emits `PoolCreatorChanged`, so indexers can follow tier-rate and fee-recipient changes without polling accounts.
- `SyncPool` emits `PoolSynced` with the reserves and the accumulator after the advance.
- `Pool` grows two fields; `pina migrations create` recorded the automatic transition and the generated clients, IDL, and ABI layout test are regenerated.
