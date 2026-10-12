---
amm: user
---

# A slightly cheaper, slightly smaller program

## User impact

Swaps, deposits, and withdrawals each cost 10–25 fewer compute units, and the deployed program is about 0.5 KB smaller, which lowers its rent. Every processor now builds the pool PDA's signer through one shared helper instead of a compiled copy per call site. Nothing about the math, fees, or account layout changed.
