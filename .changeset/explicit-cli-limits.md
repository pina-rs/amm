---
amm: breaking
---

# Require explicit absolute CLI limits

Replace simulation-derived slippage bounds with required caller-provided minimum output or maximum input for swaps and two explicit token limits for deposits and withdrawals. Remove `--slippage-bps`; keep swap quotes advisory and reject events describing another swap. Command results report submitted limits rather than simulated execution amounts.
