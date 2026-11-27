---
amm: patch
---

# Update Pina to 0.25.1

Track the Pina 0.25.1 release so the program, CPI crate, clients, CLI and Surfpool tests build against the current framework, and derive the program-data PDA at compile time with `find_program_address_const`. Compute units fall 21.0% across the 11 measured instructions (87,868 to 69,398 CU), and the deployed binary shrinks by 16 bytes (92,888 to 92,872 bytes, -0.02%) relative to 0.24.0.
