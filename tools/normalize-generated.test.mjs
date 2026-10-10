import assert from "node:assert/strict";
import { test } from "node:test";

import { normalizeDartHashes } from "./normalize-generated.mjs";

test("leaves hashes of up to 20 fields alone", () => {
	const fields = Array.from({ length: 20 }, (_, index) => `f${index}`).join(
		", ",
	);
	const source = `int get hashCode => Object.hash(${fields});`;
	assert.equal(normalizeDartHashes(source), source);
});

test("rewrites hashes of more than 20 fields to Object.hashAll", () => {
	const fields = Array.from({ length: 28 }, (_, index) => `f${index}`).join(
		", ",
	);
	assert.equal(
		normalizeDartHashes(`Object.hash(${fields})`),
		`Object.hashAll([${fields}])`,
	);
});
