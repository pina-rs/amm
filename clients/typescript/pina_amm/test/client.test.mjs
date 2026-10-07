// Smoke tests against the built bundle: run `pnpm build` first.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
	getSwapExactInInstructionDataEncoder,
	PINA_AMM_PROGRAM_ADDRESS,
	SWAP_EXACT_IN_DISCRIMINATOR,
} from "../dist/index.js";

test("exports the deployed program address", () => {
	assert.equal(
		PINA_AMM_PROGRAM_ADDRESS,
		"pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV",
	);
});

test("encodes swap data as discriminator then little-endian fields", () => {
	const bytes = getSwapExactInInstructionDataEncoder().encode({
		amountIn: 1_000n,
		minimumAmountOut: 990n,
	});
	assert.equal(bytes.length, 17);
	assert.equal(bytes[0], SWAP_EXACT_IN_DISCRIMINATOR);
	assert.equal(
		new DataView(bytes.buffer, bytes.byteOffset).getBigUint64(1, true),
		1_000n,
	);
	assert.equal(
		new DataView(bytes.buffer, bytes.byteOffset).getBigUint64(9, true),
		990n,
	);
});
