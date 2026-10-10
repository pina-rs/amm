# Performance

The Pina AMM is built to be cheap to call and cheap to deploy.

## Compute units

Measured on the real SBF artifact by the end-to-end suite (`compute_units_stay_within_budget` in `programs/pina_amm/tests/surfpool`). The numbers include the token-program CPIs each instruction makes.

| Instruction    | Compute units | CPIs                | CI ceiling |
| -------------- | ------------- | ------------------- | ---------- |
| `SwapExactIn`  | ~4,700        | 2 transfers         | 6,000      |
| `SwapExactOut` | ~4,800        | 2 transfers         | 6,000      |
| `Deposit`      | ~5,500        | 2 transfers, 1 mint | 6,500      |
| `Withdraw`     | ~5,300        | 1 burn, 2 transfers | 6,500      |
| `CreatePool`   | 38,000–45,000 | creates 5 accounts  | 60,000     |

The suite fails if any instruction exceeds its ceiling, so a regression is caught in the pull request that introduces it. `CreatePool`'s cost varies because it searches for four canonical PDA bumps whose cost depends on the addresses.

## Continuous benchmarking

Every pull request that touches the program, the clients, or the harness is benchmarked automatically. The `performance` workflow builds the real SBF binary for the pull request **and** for its base, runs the all-instructions journey three times against each binary on an offline Surfnet, and posts one consolidated comment on the pull request:

- the deployed binary size, base against head;
- every instruction's compute units, base against head, as the median of three runs.

The journey uses fixed keypair seeds for every address that feeds a PDA derivation, so identical binaries measure identical compute units and the comparison shows program changes, not address luck.

The policy in `scripts/benchmark-policy.json` gates the pull request:

- any increase in the deployed binary's size fails the check;
- any instruction whose compute units rise by more than **2%** fails the check;
- an instruction that loses its measurement fails the check;
- a new instruction is reported as a new baseline and never blocks.

A regression blocks auto-merge until the pull request carries the `performance-approved` label — the maintainer's explicit permission for that trade. Re-run the numbers locally with:

```sh
devenv shell -- pnpm exec tsx scripts/benchmark.ts --out target/perf/head
devenv shell -- pnpm exec tsx scripts/compare-benchmarks.ts \
  --base <base-dir> --head target/perf/head \
  --markdown report.md --json report.json
```

The per-instruction ceilings in the table above stay in the end-to-end suite: they catch a regression even when the workflow is skipped, for example on a docs-only pull request that shares the branch.

## Program size

About **81 KB** for the deployed `pina_amm.so`, built with fat LTO, one codegen unit, and `opt-level = 3`. Rent for the program-data account scales with this size.

The size-optimised profiles were measured and rejected:

| Profile           | Size  | `SwapExactIn`    | Result                                        |
| ----------------- | ----- | ---------------- | --------------------------------------------- |
| `opt-level = 3`   | 81 KB | ~4,700 CU        | Used                                          |
| `opt-level = "s"` | 86 KB | —                | Larger than the speed profile                 |
| `opt-level = "z"` | 77 KB | ~6,000 CU (+27%) | Every swap pays for a 6% one-time rent saving |

A market maker is deployed once and called on every swap, so it uses the speed profile. `pina build` applies that profile itself; build with `pina build --no-size-profile` to try another.

## Where the savings come from

| Choice                                   | Effect                                                                                                           |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Typed loads instead of PDA re-derivation | A pool is trusted because the program owns it and its discriminator matches; only `CreatePool` derives addresses |
| Stored vault and LP-mint addresses       | Validating an account is a 32-byte comparison, not a hash                                                        |
| Reserves derived from vault balances     | No reserve fields to write on every swap                                                                         |
| Plain `Transfer` CPIs                    | No mint accounts or decimals in the swap account list; allowed Token-2022 extensions never need them             |
| Fees accrued, not transferred            | A swap makes exactly two CPIs                                                                                    |
| `u128` arithmetic only                   | Reserves are `u64`, so every product fits `u128`; no 256-bit math                                                |
| Bit-by-bit integer square root           | No software division per iteration on SBF                                                                        |
| Pina's dispatch entrypoint               | Routes on the discriminator before parsing any account                                                           |
| Pool PDA as the vault authority          | No global authority account in every instruction                                                                 |

## Measuring yourself

```sh
devenv shell build:program
devenv shell test:surfpool    # prints "<instruction>: <units> CU (budget <limit>)"
pina profile target/deploy/pina_amm.so   # static compute-unit analysis
```
