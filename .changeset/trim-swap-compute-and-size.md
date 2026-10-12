---
amm: fix
---

# Trim the deployed program and every instruction's compute

Compiles the pool PDA's signer-seed builder once instead of a copy per call site: the deployed binary shrinks by 520 bytes and swaps, deposits, and withdrawals each get slightly cheaper. No account, instruction, event, or wire format changes.
