---
amm:
  bump: minor
  type: feat
  version: "0.1.0"
---

# Launch the Pina AMM program, clients, and CLI

The first release of the Pina AMM: a constant-product market maker built with Pina 0.23.

- **Program** (`pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV`): fee tiers gated by the upgrade authority, permissionless pool creation with optional pool-creator restriction, exact-input and exact-output swaps, proportional liquidity, protocol and creator fee accrual and collection, and Token-2022 support for transfer-neutral extensions. Swaps cost about 4,700 compute units.
- **`pina_amm_client`**, **`pina_amm_cpi`**, **`@pina-rs/amm`**, and **`pina_amm`** (Dart): generated from the program IDL with the version-0 migration baseline.
- **`pina_amm_cli`**: the `pina-amm` binary, which quotes by simulation and applies slippage limits.

An end-to-end Surfpool suite drives every instruction, the generated Rust client, and the CLI against the compiled program, and enforces compute-unit ceilings.
