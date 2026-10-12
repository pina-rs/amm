---
amm: major
---

# Deploy under the mainnet program identity

The program's address moves from `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV` to `pAMMzVPRwtrxQs2CjLTUUpfCAp9XdvVmwwVhpAuCobi` before its first mainnet deployment.

- `declare_id!` was synced with `pina keys sync`, and `pina migrations create` rebound the unpublished migration history (no publication receipts existed) to the new ID.
- The TypeScript, Dart, Rust, and CPI clients were regenerated: `PINA_AMM_PROGRAM_ADDRESS`, `PINA_AMM_ID`, the event source address, and every PDA helper's default program address now name the new ID, so every config, pool, LP mint, and vault PDA changes with it.
- Integrators that hard-code the old ID, or CPI callers that pin an older `pina_amm_cpi`, must update before calling the deployed program.
