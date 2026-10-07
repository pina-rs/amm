# TypeScript client

`@pina-rs/amm` is generated from the program's IDL for [`@solana/kit`](https://github.com/anza-xyz/kit). It ships ESM and CommonJS builds with bundled type declarations.

```sh
pnpm add @pina-rs/amm @solana/kit
# Optional, for token program addresses and associated token accounts:
pnpm add @solana-program/token
```

## What is exported

| Kind         | Examples                                                                                   |
| ------------ | ------------------------------------------------------------------------------------------ |
| Program      | `PINA_AMM_PROGRAM_ADDRESS`, `PinaAmmInstruction`, `identifyPinaAmmInstruction`             |
| Instructions | `getSwapExactInInstruction`, `getCreatePoolInstructionAsync`, `getDepositInstruction`, ... |
| Data codecs  | `getSwapExactInInstructionDataEncoder`, `getPoolDecoder`, ...                              |
| Accounts     | `fetchPool`, `fetchMaybePool`, `fetchAmmConfig`, `decodePool`, ...                         |
| PDAs         | `findAmmConfigPda`, `findPoolPda`, `findPoolVaultPda`, `findPoolLpMintPda`                 |
| Events       | `parsePinaAmmEventsFromLogs`, `decodeSwappedEvent`, ...                                    |
| Errors       | `PINA_AMM_ERROR__SLIPPAGE_EXCEEDED`, `getPinaAmmErrorMessage`, `isPinaAmmError`            |

## Find and read a pool

```ts
import { fetchPool, findAmmConfigPda, findPoolPda } from "@pina-rs/amm";
import { type Address, createSolanaRpc, getAddressEncoder } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");

/** Sort two mints by their 32 address bytes, the order the program uses. */
function sortMints(a: Address, b: Address): [Address, Address] {
	const encoder = getAddressEncoder();
	const left = encoder.encode(a);
	const right = encoder.encode(b);
	for (let index = 0; index < 32; index += 1) {
		if (left[index] !== right[index]) {
			return left[index]! < right[index]! ? [a, b] : [b, a];
		}
	}
	throw new Error("a pool needs two different mints");
}

const [ammConfig] = await findAmmConfigPda({ index: 0 });
const [mint0, mint1] = sortMints(usdcMint, solMint);
const [poolAddress] = await findPoolPda({ ammConfig, mint0, mint1 });

const pool = await fetchPool(rpc, poolAddress);
console.log(pool.data.lpSupply, pool.data.tradeFeeRate);
```

`fetchPool` throws when the account is missing and checks the discriminator and schema version. It does **not** check the owner; `fetchPool` reads the address you derived from the program, which is enough. If you decode an account from an untrusted address, compare `account.programAddress` with `PINA_AMM_PROGRAM_ADDRESS` first.

## Quote a swap off chain

Reserves are the vault balances minus the fees the pool has accrued:

```ts
import { fetchToken } from "@solana-program/token";

const [vault0, vault1] = await Promise.all([
	fetchToken(rpc, pool.data.vault0),
	fetchToken(rpc, pool.data.vault1),
]);
const reserve0 = vault0.data.amount - pool.data.protocolFees0 -
	pool.data.creatorFees0;
const reserve1 = vault1.data.amount - pool.data.protocolFees1 -
	pool.data.creatorFees1;

const D = 1_000_000n;
const ceilDiv = (a: bigint, b: bigint) => (a + b - 1n) / b;

/** Exact-input quote when the creator fee is taken from the input. */
function quoteExactIn(
	amountIn: bigint,
	reserveIn: bigint,
	reserveOut: bigint,
): bigint {
	const t = BigInt(pool.data.tradeFeeRate);
	const c = BigInt(pool.data.creatorFeeRate);
	const fee = ceilDiv(amountIn * (t + c), D);
	const net = amountIn - fee;
	return (net * reserveOut) / (reserveIn + net);
}
```

[math.md](math.md) has the formula for every creator fee mode and for exact-output swaps. When in doubt, simulate: the program's own `Swapped` event is the authoritative quote.

## Swap

```ts
import { getSwapExactInInstruction } from "@pina-rs/amm";
import {
	findAssociatedTokenPda,
	getCreateAssociatedTokenIdempotentInstructionAsync,
	TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";
import {
	appendTransactionMessageInstructions,
	createTransactionMessage,
	pipe,
	sendAndConfirmTransactionFactory,
	setTransactionMessageFeePayerSigner,
	setTransactionMessageLifetimeUsingBlockhash,
	signTransactionMessageWithSigners,
} from "@solana/kit";

const amountIn = 1_000_000n;
const quote = quoteExactIn(amountIn, reserve0, reserve1);
const minimumAmountOut = (quote * (10_000n - 50n)) / 10_000n; // 0.5%

const [inputToken] = await findAssociatedTokenPda({
	owner: wallet.address,
	mint: pool.data.mint0,
	tokenProgram: TOKEN_PROGRAM_ADDRESS,
});
const [outputToken] = await findAssociatedTokenPda({
	owner: wallet.address,
	mint: pool.data.mint1,
	tokenProgram: TOKEN_PROGRAM_ADDRESS,
});

const instructions = [
	await getCreateAssociatedTokenIdempotentInstructionAsync({
		payer: wallet,
		owner: wallet.address,
		mint: pool.data.mint1,
	}),
	getSwapExactInInstruction({
		trader: wallet,
		pool: poolAddress,
		inputToken,
		outputToken,
		inputVault: pool.data.vault0,
		outputVault: pool.data.vault1,
		inputTokenProgram: TOKEN_PROGRAM_ADDRESS,
		outputTokenProgram: TOKEN_PROGRAM_ADDRESS,
		amountIn,
		minimumAmountOut,
	}),
];

const { value: blockhash } = await rpc.getLatestBlockhash().send();
const transaction = await signTransactionMessageWithSigners(
	pipe(
		createTransactionMessage({ version: 0 }),
		(message) => setTransactionMessageFeePayerSigner(wallet, message),
		(message) =>
			setTransactionMessageLifetimeUsingBlockhash(blockhash, message),
		(message) => appendTransactionMessageInstructions(instructions, message),
	),
);
await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(transaction, {
	commitment: "confirmed",
});
```

To sell token 1 instead, swap `inputToken`/`outputToken`, `inputVault`/`outputVault`, and the token programs. For a Token-2022 mint, pass `TOKEN_2022_PROGRAM_ADDRESS` from `@solana-program/token-2022` as that side's token program.

`getSwapExactOutInstruction` takes `amountOut` and `maximumAmountIn` instead.

## Create a pool

```ts
import {
	findPoolLpMintPda,
	findPoolVaultPda,
	getCreatePoolInstructionAsync,
} from "@pina-rs/amm";

const [lpMint] = await findPoolLpMintPda({ pool: poolAddress });
const [vault0] = await findPoolVaultPda({ pool: poolAddress, mint: mint0 });
const [vault1] = await findPoolVaultPda({ pool: poolAddress, mint: mint1 });
const [lpOwnerToken] = await findAssociatedTokenPda({
	owner: wallet.address,
	mint: lpMint,
	tokenProgram: TOKEN_PROGRAM_ADDRESS,
});

const createPool = await getCreatePoolInstructionAsync({
	payer: wallet,
	depositor: wallet,
	poolCreatorAuthority: wallet, // any signer for an open tier
	ammConfig,
	mint0,
	mint1,
	lpMint,
	vault0,
	vault1,
	depositorToken0,
	depositorToken1,
	lpOwner: wallet.address,
	lpOwnerToken,
	tokenProgram0: TOKEN_PROGRAM_ADDRESS,
	tokenProgram1: TOKEN_PROGRAM_ADDRESS,
	amount0: 1_000_000_000n,
	amount1: 2_000_000_000n,
	creator: wallet.address,
	creatorFeeMode: 0,
});
```

The async builder derives `pool` for you; the LP program, associated token program, and system program have defaults.

## Liquidity

```ts
import { getDepositInstruction, getWithdrawInstruction } from "@pina-rs/amm";

const deposit = getDepositInstruction({
	owner: wallet,
	pool: poolAddress,
	vault0: pool.data.vault0,
	vault1: pool.data.vault1,
	lpMint: pool.data.lpMint,
	ownerToken0,
	ownerToken1,
	ownerLpToken,
	tokenProgram0: TOKEN_PROGRAM_ADDRESS,
	tokenProgram1: TOKEN_PROGRAM_ADDRESS,
	lpAmount: 1_000_000n,
	maximumAmount0: maxIn0,
	maximumAmount1: maxIn1,
});
```

Depositing `lp` shares costs `ceil(reserve * lp / lpSupply)` of each token, so compute `maximumAmount*` from the reserves plus your tolerance. `getWithdrawInstruction` mirrors it with `minimumAmount0` and `minimumAmount1`.

## Events

```ts
import { parsePinaAmmEventsFromLogs } from "@pina-rs/amm";

const { value } = await rpc.getTransaction(signature, {
	maxSupportedTransactionVersion: 0,
}).send();
for (
	const event of parsePinaAmmEventsFromLogs(value?.meta?.logMessages ?? [])
) {
	if (event.name === "swapped") {
		console.log(event.data.amountIn, event.data.amountOut, event.data.tradeFee);
	}
}
```

Always pass every log line of one transaction, in order.

## Errors

```ts
import {
	getPinaAmmErrorMessage,
	isPinaAmmError,
	PINA_AMM_ERROR__SLIPPAGE_EXCEEDED,
} from "@pina-rs/amm";

try {
	await send(transaction);
} catch (error) {
	if (
		isPinaAmmError(error, transactionMessage, PINA_AMM_ERROR__SLIPPAGE_EXCEEDED)
	) {
		// re-quote and retry
	}
}
```
