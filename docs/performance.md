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

## Program size

About **81 KB** for the deployed `pina_amm.so`, built with fat LTO, one codegen unit, and `opt-level = 3`. Rent for the program-data account scales with this size.

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
