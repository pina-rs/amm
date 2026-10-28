---
amm: breaking
---

# Preserve the full time-weighted price integral

The canonical price accumulator in `Pool` and `PoolSynced` now combines `price_0_cumulative_last` (low) with `price_0_cumulative_high_last` (high) into a 256-bit value. All generated clients expose both limbs. Integrators must reconstruct the full value before subtracting samples; every supported reserve ratio and timestamp window has a delta below `2^192`, so the complete integral is recoverable without halting trading. Take both samples after migration, which initializes the high limb to zero and cannot recover previously lost cycles.
