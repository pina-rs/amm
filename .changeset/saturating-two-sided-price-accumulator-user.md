---
amm: user
---

# Markets survive extreme prices and quote their average price both ways

## User impact

Two changes to the price history every market keeps. First, a market that drifts to an extreme price no longer locks up: previously, if a market sat untouched at a very lopsided price long enough, every later trade failed outright until someone added liquidity by hand. Now the market keeps trading; only its long-run price average in the extreme direction stops being meaningful, and it reads back as the maximum value instead of a usable number. Second, the time-weighted average price is now tracked in both directions exactly, so an oracle can quote either token against the other without a lossy conversion. Trading costs are otherwise unchanged.
