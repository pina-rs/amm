import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { JOURNEY_CLOCK_PIN, pinJourneyClock } from "../scripts/benchmark.ts";

const journeyPath = new URL(
	"../programs/pina_amm/tests/surfpool/src/journeys.rs",
	import.meta.url,
);

function unpin(source: string): string {
	return source.replaceAll("\t\tadvance_clock(&h);\n", "").replace(
		JOURNEY_CLOCK_PIN,
		"",
	);
}

test("the committed journey is the clock pin applied to the previous journey", () => {
	const pinned = readFileSync(journeyPath, "utf8");
	const calls = pinned.match(/\tadvance_clock\(&h\);/g) ?? [];
	assert.equal(calls.length, 5);
	assert.equal(pinned.includes(JOURNEY_CLOCK_PIN), true);

	const unpinned = unpin(pinned);
	assert.equal(unpinned.includes("advance_clock"), false);
	assert.equal(pinJourneyClock(unpinned), pinned);
	assert.equal(pinJourneyClock(pinned), pinned);
});

test("pinJourneyClock leaves a journey alone when an anchor is missing", () => {
	const pinned = readFileSync(journeyPath, "utf8");
	const broken = unpin(pinned).replace(
		"&[withdraw_instruction(",
		"&[withdraw_instruction_renamed(",
	);
	assert.equal(pinJourneyClock(broken), undefined);
});
