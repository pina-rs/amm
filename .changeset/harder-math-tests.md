---
amm: none
---

# Test-only hardening: exact-rounding proofs, mutation gaps, invariant fuzzing

The math module's tests now pin the exact rounding of every quoted amount by cross-multiplication — fees as exact ceilings, outputs as exact floors, shares as exact floors — closing the coverage gaps a first cargo-mutants run surfaced: exact-output fee amounts were never checked numerically, the constant-product guard's rejection path was never exercised, and the zero-sided liquidity amounts were untested. A seeded Surfpool fuzzer additionally drives one pool through random swap, deposit, withdraw, sync, and collection sequences, asserting the security model's global invariants after every transaction, and an extreme-ratio pool now proves end to end that trading continues after the accumulator wraps. No program behavior changes.
