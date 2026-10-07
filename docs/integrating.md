# Integrating the Pina AMM

Pick the client that matches where your code runs:

| You are writing                          | Use                               | Guide                            |
| ---------------------------------------- | --------------------------------- | -------------------------------- |
| A web app, bot, or backend in TypeScript | `@pina-rs/amm` with `@solana/kit` | [typescript.md](typescript.md)   |
| A Flutter app or Dart backend            | `pina_amm` with `solana_kit`      | [dart.md](dart.md)               |
| A Rust service, keeper, or test          | `pina_amm_client`                 | [rust-client.md](rust-client.md) |
| Another on-chain program                 | `pina_amm_cpi`                    | [cpi.md](cpi.md)                 |
| Scripts and operations                   | the `pina-amm` CLI                | [cli.md](cli.md)                 |

All clients are generated from the same IDL, so names match across languages (`swapExactIn` in TypeScript and Dart is `SwapExactIn` in Rust).

## Addresses you will need

| Address        | How to get it                                                                      |
| -------------- | ---------------------------------------------------------------------------------- |
| Program        | `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV`, exported by every client            |
| Fee tier       | `findAmmConfigPda({ index })`                                                      |
| Pool           | `findPoolPda({ ammConfig, mint0, mint1 })`, with the mints sorted by their bytes   |
| Vaults         | stored on the pool as `vault0` and `vault1`, or `findPoolVaultPda({ pool, mint })` |
| LP mint        | stored on the pool as `lpMint`, or `findPoolLpMintPda({ pool })`                   |
| Token programs | the owner of each mint account                                                     |

Sorting mints: compare the 32 address bytes, not the base58 strings. Base58 order differs from byte order. Every client exposes the address encoder needed to compare bytes; the CLI sorts for you.

## A swap, step by step

1. **Fetch the pool** and confirm its owner is the AMM program. Generated decoders only check the discriminator, so the owner check is yours.
2. **Pick the direction.** Selling token 0 means `inputVault = pool.vault0` and `outputVault = pool.vault1`; selling token 1 swaps them.
3. **Quote.** Either simulate the swap with an open limit (`minimumAmountOut = 0`) and read the `Swapped` event, or compute the quote yourself from [math.md](math.md) using the vault balances minus the pool's accrued fees.
4. **Apply slippage** to the quote: `minimumAmountOut = floor(quote * (10_000 - bps) / 10_000)` for exact-input swaps, `maximumAmountIn = ceil(quote * (10_000 + bps) / 10_000)` for exact-output swaps.
5. **Make sure the output account exists.** Prepend an idempotent associated-token-account creation if needed.
6. **Send.** If the pool moved more than your tolerance, the program fails with `SlippageExceeded` and nothing changes.

Simulation quoting (step 3, first option) is what the CLI does: it can never disagree with the program because it _is_ the program.

## Wrapped SOL

The AMM trades SPL tokens only. To trade SOL, use the wrapped SOL mint (`So11111111111111111111111111111111111111112`): create or reuse a wrapped SOL token account, transfer lamports into it and call `SyncNative`, swap, then close it to unwrap. All of this fits in one transaction alongside the swap.

## Compute budget

| Instruction                    | Measured         | Suggested limit |
| ------------------------------ | ---------------- | --------------- |
| `SwapExactIn` / `SwapExactOut` | ~4,800 CU        | 10,000          |
| `Deposit` / `Withdraw`         | ~5,500 CU        | 10,000          |
| `CreatePool`                   | 38,000–45,000 CU | 80,000          |

Add the cost of anything else in the transaction, such as associated-token-account creation (~25,000 CU). Requesting a tight limit lowers the priority fee you pay. See [performance.md](performance.md).

## Reading events

Every state change emits an event: `PoolCreated`, `Swapped`, `LiquidityChanged`, and `FeesCollected`. Pass the **complete, ordered** log messages of a transaction to `parsePinaAmmEventsFromLogs`. It follows the runtime's invoke frames and only returns records the AMM itself wrote, so a program that calls the AMM, or that the AMM calls, cannot forge an AMM event. The per-event `parse*FromLog` helpers skip that attribution and are only safe for bytes you already know came from the AMM.

## Errors

Program errors arrive as `Custom(code)`. Every client exports the codes and messages; the table is in [instructions.md](instructions.md#errors). The most common in practice:

| Error                           | Usual cause                                                               |
| ------------------------------- | ------------------------------------------------------------------------- |
| `SlippageExceeded` (9)          | The pool moved between quote and send; re-quote                           |
| `PoolAccountMismatch` (5)       | A vault from another pool, or vaults in the wrong order for the direction |
| `UnsupportedMint` (4)           | A Token-2022 mint with a transfer fee, hook, or other blocked extension   |
| `PoolCreatorNotAuthorized` (14) | Creating a pool in a tier reserved for another program                    |
