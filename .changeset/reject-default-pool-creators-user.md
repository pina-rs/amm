---
amm: user
---

# Pools can no longer name an empty creator address

## User impact

Fees that would have accrued to an address nobody controls are now impossible to create: naming the unset default address as a pool creator fails with `DefaultCreator`, and so does handing an existing pool's creator rights to it.
