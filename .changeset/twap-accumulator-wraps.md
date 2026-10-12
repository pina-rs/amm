---
amm: breaking
---

# Wrap the price accumulator instead of halting the pool

`advance_price_accumulator` now wraps modulo `2^128`, like Uniswap V2, instead of failing with `MathOverflow`. The accumulator only ever grows, so a checked sum would have failed every swap and `SyncPool` permanently once a pool with an extreme price ratio had accumulated enough time, with no way to recover short of a program upgrade. Integrators must subtract samples with wrapping arithmetic; `docs/math.md` shows how. The function no longer returns a `Result`.
