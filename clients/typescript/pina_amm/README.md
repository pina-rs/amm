# `@pina-rs/amm`

TypeScript client for the [Pina AMM](https://github.com/pina-rs/amm), a permissionless constant-product market maker on Solana, for [`@solana/kit`](https://github.com/anza-xyz/kit). Generated from the program's IDL.

```sh
pnpm add @pina-rs/amm @solana/kit
```

```ts
import { fetchPool, getSwapExactInInstruction } from "@pina-rs/amm";

const { data: pool } = await fetchPool(rpc, poolAddress);
const instruction = getSwapExactInInstruction({
	trader: wallet,
	pool: poolAddress,
	inputToken,
	outputToken,
	inputVault: pool.vault0,
	outputVault: pool.vault1,
	inputTokenProgram: TOKEN_PROGRAM_ADDRESS,
	outputTokenProgram: TOKEN_PROGRAM_ADDRESS,
	amountIn: 1_000_000n,
	minimumAmountOut: 990_000n,
});
```

Full guide: [docs/typescript.md](https://github.com/pina-rs/amm/blob/main/docs/typescript.md).
