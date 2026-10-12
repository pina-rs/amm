---
amm: user
---

# Average prices can no longer be rewritten after the fact

## User impact

A pool's time-weighted average price now only counts each price for the time it actually stood. Sending tokens straight into a pool and syncing it can no longer change the average for time that has already passed, so integrations that rely on the average are much harder to manipulate. New fee tiers now announce themselves with an event, and the program publishes its security contact details so researchers can reach the maintainers privately.
