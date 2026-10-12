import { readFileSync } from "node:fs";

/**
 * Drives every Pina AMM instruction against one live Surfnet, end to end,
 * through the generated TypeScript client.
 *
 * The Rust test `typescript_client_runs_every_instruction` starts the
 * Surfnet, funds the signers, creates the token fixtures, derives every
 * address, and then runs this script:
 *
 * ```sh
 * pnpm --dir clients/typescript/pina_amm run journey -- <rpc-url> <fixtures.json>
 * ```
 *
 * The script asserts that the client's own PDA helpers reproduce the
 * Rust-derived addresses before sending anything, then runs all ten
 * instructions in order.
 */

import {
	type Address,
	airdropFactory,
	appendTransactionMessageInstructions,
	createDefaultRpcTransport,
	createKeyPairSignerFromBytes,
	createSolanaRpcFromTransport,
	createTransactionMessage,
	generateKeyPairSigner,
	getBase64EncodedWireTransaction,
	type Instruction,
	type KeyPairSigner,
	pipe,
	setTransactionMessageFeePayerSigner,
	setTransactionMessageLifetimeUsingBlockhash,
	signTransactionMessageWithSigners,
} from "@solana/kit";

import {
	findAmmConfigPda,
	findPoolLpMintPda,
	findPoolPda,
	findPoolVaultPda,
	getCollectCreatorFeesInstruction,
	getCollectProtocolFeesInstruction,
	getCreateConfigInstruction,
	getCreatePoolInstruction,
	getDepositInstruction,
	getSetPoolCreatorInstruction,
	getSwapExactInInstruction,
	getSwapExactOutInstruction,
	getUpdateConfigInstruction,
	getWithdrawInstruction,
} from "../src/index.js";

/** The original SPL Token program, the token program every journey mint uses. */
const TOKEN_PROGRAM_ADDRESS =
	"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Parameters<
		typeof getDepositInstruction
	>[0]["tokenProgram0"];

interface JourneyFixtures {
	adminKey: number[];
	programData: string;
	ownerKey: number[];
	configIndex: number;
	mint0: string;
	mint1: string;
	ownerToken0: string;
	ownerToken1: string;
	ownerLpToken: string;
	addresses: {
		config: string;
		pool: string;
		lpMint: string;
		vault0: string;
		vault1: string;
	};
}

function assertEqual(actual: string, expected: string, label: string): void {
	if (actual !== expected) {
		throw new Error(
			`${label}: client derived ${actual}, fixtures say ${expected}`,
		);
	}
}

async function main(): Promise<void> {
	const flag = (name: string): string => {
		const index = process.argv.indexOf(name);
		const value = index === -1 ? undefined : process.argv[index + 1];
		if (value === undefined) {
			throw new Error(
				`usage: journey --rpc <url> --fixtures <file> (missing ${name})`,
			);
		}
		return value;
	};
	const rpcUrl = flag("--rpc");
	const fixturesPath = flag("--fixtures");
	const fixtures: JourneyFixtures = JSON.parse(
		readFileSync(fixturesPath, "utf8"),
	) as JourneyFixtures;

	const rpc = createSolanaRpcFromTransport(
		createDefaultRpcTransport({ url: rpcUrl }),
	);

	// The Rust side funds both signers before the journey starts.
	const admin = await createKeyPairSignerFromBytes(
		new Uint8Array(fixtures.adminKey),
	);
	const owner = await createKeyPairSignerFromBytes(
		new Uint8Array(fixtures.ownerKey),
	);

	// The client must derive exactly the addresses the Rust side derived.
	assertEqual(
		(await findAmmConfigPda({ index: fixtures.configIndex }))[0],
		fixtures.addresses.config,
		"amm config PDA",
	);
	assertEqual(
		(
			await findPoolPda({
				ammConfig: fixtures.addresses.config as Address,
				mint0: fixtures.mint0 as Address,
				mint1: fixtures.mint1 as Address,
			})
		)[0],
		fixtures.addresses.pool,
		"pool PDA",
	);
	assertEqual(
		(await findPoolLpMintPda({ pool: fixtures.addresses.pool as Address }))[0],
		fixtures.addresses.lpMint,
		"LP mint PDA",
	);
	assertEqual(
		(
			await findPoolVaultPda({
				pool: fixtures.addresses.pool as Address,
				mint: fixtures.mint0 as Address,
			})
		)[0],
		fixtures.addresses.vault0,
		"pool vault 0 PDA",
	);

	const config = fixtures.addresses.config as Address;
	const pool = fixtures.addresses.pool as Address;
	const lpMint = fixtures.addresses.lpMint as Address;
	const vault0 = fixtures.addresses.vault0 as Address;
	const vault1 = fixtures.addresses.vault1 as Address;
	const mint0 = fixtures.mint0 as Address;
	const mint1 = fixtures.mint1 as Address;
	const ownerToken0 = fixtures.ownerToken0 as Address;
	const ownerToken1 = fixtures.ownerToken1 as Address;
	const ownerLpToken = fixtures.ownerLpToken as Address;
	const tokenProgram = TOKEN_PROGRAM_ADDRESS;

	async function sendAll(
		payer: KeyPairSigner,
		instructions: Instruction[],
	): Promise<void> {
		const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
		const message = pipe(
			createTransactionMessage({ version: 0 }),
			(message) => setTransactionMessageFeePayerSigner(payer, message),
			(message) =>
				setTransactionMessageLifetimeUsingBlockhash(
					{
						blockhash: latestBlockhash.blockhash,
						lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
					},
					message,
				),
			(message) => appendTransactionMessageInstructions(instructions, message),
		);
		const transaction = await signTransactionMessageWithSigners(message);
		const signature = await rpc
			.sendTransaction(getBase64EncodedWireTransaction(transaction), {
				encoding: "base64",
			})
			.send();
		for (let attempt = 0; attempt < 60; attempt += 1) {
			const statuses = await rpc.getSignatureStatuses([signature]).send();
			const status = statuses.value[0];
			if (
				status?.confirmationStatus === "confirmed" ||
				status?.confirmationStatus === "finalized"
			) {
				if (status.err !== null) {
					throw new Error(
						`transaction ${signature} failed: ${JSON.stringify(status.err)}`,
					);
				}
				return;
			}
			if (status?.err !== null && status?.err !== undefined) {
				throw new Error(
					`transaction ${signature} failed: ${JSON.stringify(status.err)}`,
				);
			}
			await new Promise((resolve) => setTimeout(resolve, 100));
		}
		throw new Error(`transaction ${signature} never confirmed`);
	}

	// 1. CreateConfig — the upgrade authority opens a fee tier.
	await sendAll(
		admin,
		[
			getCreateConfigInstruction({
				payer: admin,
				upgradeAuthority: admin,
				programData: fixtures.programData as Address,
				ammConfig: config,
				index: fixtures.configIndex,
				tradeFeeRate: 2_500,
				protocolFeeRate: 200_000,
				creatorFeeRate: 500,
				authority: admin.address,
				poolCreatorAuthority: "11111111111111111111111111111111" as Address,
			}),
		],
	);

	// 2. UpdateConfig — re-state the same rates.
	await sendAll(
		admin,
		[
			getUpdateConfigInstruction({
				authority: admin,
				ammConfig: config,
				newAuthority: admin.address,
				tradeFeeRate: 2_500,
				protocolFeeRate: 200_000,
				creatorFeeRate: 500,
			}),
		],
	);

	// 3. CreatePool — the owner makes the first deposit and receives the LP.
	await sendAll(
		owner,
		[
			getCreatePoolInstruction({
				payer: owner,
				depositor: owner,
				poolCreatorAuthority: owner,
				ammConfig: config,
				mint0,
				mint1,
				pool,
				lpMint,
				vault0,
				vault1,
				depositorToken0: ownerToken0,
				depositorToken1: ownerToken1,
				lpOwner: owner,
				lpOwnerToken: ownerLpToken,
				tokenProgram0: tokenProgram,
				tokenProgram1: tokenProgram,
				amount0: 1_000_000_000n,
				amount1: 4_000_000_000n,
				creator: owner.address,
				creatorFeeMode: 0,
			}),
		],
	);

	// 4. Deposit — proportional liquidity for more LP.
	await sendAll(
		owner,
		[
			getDepositInstruction({
				owner,
				pool,
				vault0,
				vault1,
				lpMint,
				ownerToken0,
				ownerToken1,
				ownerLpToken,
				tokenProgram0: tokenProgram,
				tokenProgram1: tokenProgram,
				lpAmount: 1_000_000n,
				maximumAmount0: 0xffff_ffff_ffff_ffffn,
				maximumAmount1: 0xffff_ffff_ffff_ffffn,
			}),
		],
	);

	// 5. SwapExactIn — sell token 0 for token 1.
	await sendAll(
		owner,
		[
			getSwapExactInInstruction({
				trader: owner,
				pool,
				inputToken: ownerToken0,
				outputToken: ownerToken1,
				inputVault: vault0,
				outputVault: vault1,
				inputTokenProgram: tokenProgram,
				outputTokenProgram: tokenProgram,
				amountIn: 10_000_000n,
				minimumAmountOut: 0n,
			}),
		],
	);

	// 6. SwapExactOut — buy an exact amount of token 0 back.
	await sendAll(
		owner,
		[
			getSwapExactOutInstruction({
				trader: owner,
				pool,
				inputToken: ownerToken1,
				outputToken: ownerToken0,
				inputVault: vault1,
				outputVault: vault0,
				inputTokenProgram: tokenProgram,
				outputTokenProgram: tokenProgram,
				amountOut: 1_000_000n,
				maximumAmountIn: 0xffff_ffff_ffff_ffffn,
			}),
		],
	);

	// 7. CollectCreatorFees — the creator empties its accrued fees.
	await sendAll(
		owner,
		[
			getCollectCreatorFeesInstruction({
				creator: owner,
				pool,
				vault0,
				vault1,
				recipientToken0: ownerToken0,
				recipientToken1: ownerToken1,
				tokenProgram0: tokenProgram,
				tokenProgram1: tokenProgram,
				maximumAmount0: 0xffff_ffff_ffff_ffffn,
				maximumAmount1: 0xffff_ffff_ffff_ffffn,
			}),
		],
	);

	// 8. SetPoolCreator — hand the creator-fee rights to a fresh wallet.
	const successor = await generateKeyPairSigner();
	await sendAll(
		owner,
		[
			getSetPoolCreatorInstruction({
				creator: owner,
				pool,
				newCreator: successor.address,
			}),
		],
	);

	// 9. CollectProtocolFees — the tier authority empties the protocol share.
	await sendAll(
		admin,
		[
			getCollectProtocolFeesInstruction({
				authority: admin,
				ammConfig: config,
				pool,
				vault0,
				vault1,
				recipientToken0: ownerToken0,
				recipientToken1: ownerToken1,
				tokenProgram0: tokenProgram,
				tokenProgram1: tokenProgram,
				maximumAmount0: 0xffff_ffff_ffff_ffffn,
				maximumAmount1: 0xffff_ffff_ffff_ffffn,
			}),
		],
	);

	// 10. Withdraw — take the deposited liquidity back out.
	await sendAll(
		owner,
		[
			getWithdrawInstruction({
				owner,
				pool,
				vault0,
				vault1,
				lpMint,
				ownerToken0,
				ownerToken1,
				ownerLpToken,
				tokenProgram0: tokenProgram,
				tokenProgram1: tokenProgram,
				lpAmount: 1_000_000n,
				minimumAmount0: 0n,
				minimumAmount1: 0n,
			}),
		],
	);

	console.log("typescript journey: all ten instructions confirmed");
}

await main();
