---
amm: user
---

# Keep average prices accurate in pools with extreme ratios

## User impact

Average prices now retain the entire elapsed price history even when one token is much scarcer than the other. A continuously positive pool price can no longer appear to be zero because its accumulator ran out of room, and swaps continue to settle.
