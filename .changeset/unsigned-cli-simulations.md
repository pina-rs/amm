---
amm: fix
---

# Keep simulation transactions unsigned

Construct quote and `--simulate` transactions with placeholder signatures and a default blockhash. Fetch a recent blockhash and sign only when sending a transaction, so an RPC cannot broadcast a captured simulation with the user's authority.
