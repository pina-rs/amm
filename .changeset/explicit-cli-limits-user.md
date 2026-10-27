---
amm: user
---

# Choose the most you will pay or the least you will receive

## User impact

Swaps, deposits, and withdrawals now require absolute token limits that you choose. The RPC's simulation results can no longer weaken those limits. Replace `--slippage-bps` with `--minimum-out` or `--maximum-in` for swaps, and both `--maximum-amount-0/1` or `--minimum-amount-0/1` values for liquidity. Verify prices independently before choosing your limits; quotes remain advisory.
