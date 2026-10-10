---
amm: user
---

# Pools now track their average price over time

## User impact

Every market now keeps a running sum of its price over time, and a new permissionless `SyncPool` instruction advances it on demand. Anyone can sample that sum at two moments to compute a time-weighted average price that a single transaction cannot meaningfully move — the standard building block for pricing collateral and rewards. Nothing else about trading changes: swaps cost the same and advance the accumulator themselves.
