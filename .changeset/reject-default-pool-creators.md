---
amm: major
---

# Reject the default address as a pool creator

`CreatePool` and `SetPoolCreator` reject the default address with the new `DefaultCreator` error. Creator fees accrued to the default address could never be collected, yet they still reduce the reserves every withdrawal pays from.
