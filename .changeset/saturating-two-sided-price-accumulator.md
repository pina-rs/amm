---
amm: major
---

# Saturating two-sided price accumulator and tier-creation event

- `Pool` now tracks the time-weighted price in both directions: `price_1_cumulative_last` (Q64.64 price of token 0 in token 1) joins `price_0_cumulative_last`, each an exact integer sum so neither direction has to be derived from the other. `Swapped` and the new `PoolSynced` field report both.
- The accumulators **saturate** at `u128::MAX` instead of overflowing: a pool left idle at an extreme reserve ratio no longer fails every later swap with `MathOverflow`; trading continues and the saturated direction is readable as `u128::MAX`.
- `CreateConfig` now emits `ConfigCreated`, closing the event-coverage gap for tier creation (`ConfigUpdated` and `SetPoolCreator` already emitted theirs).
- `pina migrations create` recorded the account and event changes; generated clients and the ABI layout test are regenerated.
