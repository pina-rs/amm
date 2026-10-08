# Pina AMM

A permissionless constant-product market maker for Solana, built with [Pina](https://github.com/pina-rs/pina).

**Website and documentation:** [amm.pina.rs](https://amm.pina.rs)

Anyone can create a pool for any two tokens, trade against it, and provide liquidity. Fee tiers are curated by the program's upgrade authority, so traders and liquidity providers choose between a small set of well-known fee levels instead of reading every pool's terms. A tier can also be reserved for one pool creator — such as a launchpad program — so that program's pools cannot be front-run.

```text
Program ID  pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV
```

> **Status:** pre-release. The program has not been deployed or audited, and the interface may still change before the first `v*` tag. See [SECURITY.md](SECURITY.md).

## Why another AMM

The Pina AMM keeps the shape of Raydium's CP-Swap and removes everything an integration does not need:

- **Small and fast.** A swap costs about **4,700 compute units** including both token transfers, and the whole program is about **80 KB**. See [docs/performance.md](docs/performance.md).
- **Nothing to trust after creation.** A pool snapshots its tier's fee rates when it is created; a later tier update only affects new pools. There is no pause switch and no admin path to a pool's funds.
- **Creator fees built in.** Each tier can charge an extra creator fee, paid to the pool's creator in the token of their choosing. A launchpad can route post-launch trading income to a token's creator without a separate program.
- **Token-2022 ready.** SPL Token and Token-2022 mints are both supported, as long as their extensions cannot change how many tokens a transfer moves (metadata, groups, interest-bearing and scaled-UI-amount mints are fine; transfer fees and hooks are not).
- **Generated clients for every stack.** Rust, TypeScript, Dart/Flutter, and on-chain CPI clients are generated from the program's IDL, plus a hand-written `pina-amm` CLI.

## Packages

| Package           | Registry                                              | What it is                                                | Guide                                      |
| ----------------- | ----------------------------------------------------- | --------------------------------------------------------- | ------------------------------------------ |
| `pina_amm_client` | [crates.io](https://crates.io/crates/pina_amm_client) | Rust client: instruction builders, decoders, PDAs, events | [docs/rust-client.md](docs/rust-client.md) |
| `pina_amm_cpi`    | [crates.io](https://crates.io/crates/pina_amm_cpi)    | `no_std` client for calling the AMM from another program  | [docs/cpi.md](docs/cpi.md)                 |
| `pina_amm_cli`    | [crates.io](https://crates.io/crates/pina_amm_cli)    | The `pina-amm` command-line tool                          | [docs/cli.md](docs/cli.md)                 |
| `@pina-rs/amm`    | [npm](https://www.npmjs.com/package/@pina-rs/amm)     | TypeScript client for `@solana/kit`                       | [docs/typescript.md](docs/typescript.md)   |
| `pina_amm`        | [pub.dev](https://pub.dev/packages/pina_amm)          | Dart and Flutter client for `solana_kit`                  | [docs/dart.md](docs/dart.md)               |

The on-chain program itself lives in [`programs/pina_amm`](programs/pina_amm). It ships through `pina deploy` (see [docs/deploying.md](docs/deploying.md)), never through a package registry.

## Quick start

Install the CLI and look at a pool:

```sh
cargo install pina_amm_cli
pina-amm --url mainnet pool show --pool <POOL_ADDRESS>
```

Quote a trade, then send it with a 0.5% slippage limit:

```sh
pina-amm quote --pool <POOL> --sell <MINT> --amount-in 1000000
pina-amm swap  --pool <POOL> --sell <MINT> --amount-in 1000000 --slippage-bps 50
```

The same swap from TypeScript:

```ts
import { fetchPool, getSwapExactInInstruction } from "@pina-rs/amm";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");
const { data: pool } = await fetchPool(rpc, poolAddress);

// Sell token 0 for token 1. To sell token 1, reverse every input/output pair:
// the token accounts, the vaults, and the token programs.
const instruction = getSwapExactInInstruction({
	trader: wallet, // a TransactionSigner
	pool: poolAddress,
	inputToken: walletToken0,
	outputToken: walletToken1,
	inputVault: pool.vault0,
	outputVault: pool.vault1,
	inputTokenProgram: TOKEN_PROGRAM_ADDRESS,
	outputTokenProgram: TOKEN_PROGRAM_ADDRESS,
	amountIn: 1_000_000n,
	minimumAmountOut: 990_000n,
});
```

Every client guide has complete, runnable examples. Start with [docs/integrating.md](docs/integrating.md) to choose one.

## How it works

| Concept                | Summary                                                                       | Details                                               |
| ---------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------- |
| Fee tier (`AmmConfig`) | Trade fee, protocol share, creator fee, and optional pool-creator restriction | [docs/fees.md](docs/fees.md)                          |
| Pool (`Pool`)          | One pair under one tier, at `[b"pool", amm_config, mint_0, mint_1]`           | [docs/architecture.md](docs/architecture.md)          |
| Reserves               | Vault balance minus fees accrued for collection; donations go to LPs          | [docs/architecture.md](docs/architecture.md#reserves) |
| Pricing                | `x * y = k`, every rounding in the pool's favour                              | [docs/math.md](docs/math.md)                          |
| Liquidity              | Proportional deposits and withdrawals; 1,000 LP units locked forever          | [docs/math.md](docs/math.md#liquidity)                |
| Instructions           | Ten instructions with fixed account lists                                     | [docs/instructions.md](docs/instructions.md)          |

## Development

Everything runs inside [devenv](https://devenv.sh), which pins Rust, the Agave SBF toolchain, the Pina CLI, Node.js, Dart, and Monochange:

```sh
devenv shell install:all     # JavaScript and Dart dependencies
devenv shell build:program   # SBF artifact and IDL in target/
devenv shell test:unit       # Rust, TypeScript, and Dart unit tests
devenv shell test:surfpool   # end-to-end suite on an offline Surfnet
devenv shell lint:all        # Pina security lints, clippy, docs, formatting
devenv shell dev:website     # amm.pina.rs locally, with docs/ as its documentation
devenv shell verify:all      # everything CI runs
```

After any change to the program's accounts, instructions, or events, run `pina migrations create --project programs/pina_amm` and then `devenv shell generate:clients`. Generated sources under `clients/` are never edited by hand. [AGENTS.md](AGENTS.md) lists the full set of repository rules.

## Releases

Releases are planned with [Monochange](https://github.com/monochange/monochange). Every pull request that changes a published package carries a changeset; user-visible changes carry a second, plain-language changeset for the public release notes. [docs/releasing.md](docs/releasing.md) covers the flow, and [docs/deploying.md](docs/deploying.md) covers deploying the program and creating fee tiers.

## License

[Apache-2.0](LICENSE)
