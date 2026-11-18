#!/usr/bin/env node
// Builds the program, measures the deployed binary size, and records the
// compute units of every instruction by running the all-instructions journey
// on a real Surfnet with `PINA_CU_RECORD_FILE` set. The output directory
// holds `benchmark.json`, the single artifact `compare-benchmarks.ts`
// compares between the PR base and head.
//
// The journey moves Surfnet's clock two seconds past the pool's last oracle
// update before every instruction that observes it. `advance_price_accumulator`
// divides only after the clock has moved. Surfnet's clock otherwise follows
// wall time, so an unpinned journey measures timing luck. When the base
// journey does not yet contain the pin, this script applies the same edit
// for the measurement and restores the file afterwards. The base program
// still runs in the base test crate, so an account-layout change keeps its
// own decoder.
//
// Usage: node scripts/benchmark.ts --out <directory>

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const PROGRAM = "pina_amm";
const PROGRAM_DIR = "programs/pina_amm";
const JOURNEY_TEST = "journeys::every_instruction";
const TEST_MANIFEST = "programs/pina_amm/tests/surfpool/Cargo.toml";
const JOURNEY_SOURCE = "programs/pina_amm/tests/surfpool/src/journeys.rs";

// Text of the clock pin committed in the journey. Replayed onto a base
// journey that predates the pin, so both binaries take the division path.
export const JOURNEY_CLOCK_PIN = [
	"/// Move the cluster clock past the pool's last oracle update before an",
	"/// instruction that observes the price oracle.",
	"///",
	"/// `advance_price_accumulator` returns immediately when `now <= last_update`",
	"/// and otherwise runs a `u128` division. Surfnet's clock follows wall time, so",
	"/// without this jump an instruction's compute units depend on whether a second",
	"/// boundary fell since the previous observation. The jump is two seconds, not",
	"/// one, so it still clears `last_update` when the clock sysvar trails the",
	"/// timestamp `advance_seconds` reads. `sync_accumulates_the_time_weighted_price`",
	"/// allows the same one-second slack.",
	"fn advance_clock(h: &Harness) {",
	'\th.advance_seconds(2).expect("advance the clock");',
	"}",
	"",
	"",
].join("\n");

const CLOCK_OBSERVATIONS: ReadonlyArray<readonly [string, string]> = [
	[
		"\t\th.send(\n\t\t\t&[deposit_instruction(",
		"\t\tadvance_clock(&h);\n\t\th.send(\n\t\t\t&[deposit_instruction(",
	],
	[
		"\t\th.send(\n\t\t\t&[swap_exact_in_instruction(",
		"\t\tadvance_clock(&h);\n\t\th.send(\n\t\t\t&[swap_exact_in_instruction(",
	],
	[
		"\t\th.send(\n\t\t\t&[swap_exact_out_instruction(",
		"\t\tadvance_clock(&h);\n\t\th.send(\n\t\t\t&[swap_exact_out_instruction(",
	],
	[
		"\t\t// SyncPool advances the accumulator outside a trade.\n\t\th.send(\n",
		"\t\t// SyncPool advances the accumulator outside a trade.\n\t\tadvance_clock(&h);\n\t\th.send(\n",
	],
	[
		"\t\th.send(\n\t\t\t&[withdraw_instruction(",
		"\t\tadvance_clock(&h);\n\t\th.send(\n\t\t\t&[withdraw_instruction(",
	],
];

interface IdlInstruction {
	name: string;
	discriminators?: Array<{
		offset?: number;
		constant?: { value?: { number?: number } };
	}>;
}

interface Idl {
	program?: { instructions?: IdlInstruction[] };
}

interface Sample {
	program: string;
	discriminator: number;
	computeUnits: number;
}

interface Benchmark {
	// Unix seconds of the run.
	timestamp: number;
	binaryBytes: number;
	binarySha256: string;
	instructions: Array<{
		name: string;
		discriminator: number;
		samples: number;
		median: number;
	}>;
}

function parseArguments(): { out: string; buildRoot: string } {
	const value = (flag: string): string | undefined => {
		const index = process.argv.indexOf(flag);
		return index === -1 ? undefined : process.argv[index + 1];
	};
	const out = value("--out");
	if (out === undefined) {
		throw new Error(
			"usage: benchmark.ts --out <directory> [--build-root <checkout>]",
		);
	}
	return {
		out: resolve(out),
		buildRoot: resolve(value("--build-root") ?? "."),
	};
}

function run(
	command: string,
	args: string[],
	options: { cwd?: string; env?: Record<string, string> } = {},
): void {
	const result = spawnSync(command, args, {
		stdio: "inherit",
		cwd: options.cwd,
		env: { ...process.env, ...options.env },
	});
	if (result.status !== 0) {
		throw new Error(
			`${command} ${args.join(" ")} failed with status ${
				String(result.status)
			}`,
		);
	}
}

// Replay the journey's oracle-clock pin onto `source`.
//
// Returns `source` when the pin is already present. Returns `undefined` when
// `source` has no pin and is missing an anchor, so a partial edit cannot
// measure some instructions on the division path and others on the early
// return.
export function pinJourneyClock(source: string): string | undefined {
	if (source.includes("fn advance_clock(")) {
		return source;
	}
	const marker = "/// The journey's tier authority:";
	if (!source.includes(marker)) {
		return undefined;
	}
	let pinned = source.replace(marker, `${JOURNEY_CLOCK_PIN}${marker}`);
	for (const [from, to] of CLOCK_OBSERVATIONS) {
		if (!pinned.includes(from)) {
			return undefined;
		}
		pinned = pinned.replace(from, to);
	}
	return pinned;
}

// Pin the base journey's clock when it predates this edit, then restore the
// file. The head journey is left untouched.
function alignBaseJourneyClock(buildRoot: string): () => void {
	const basePath = resolve(buildRoot, JOURNEY_SOURCE);
	const headPath = resolve(JOURNEY_SOURCE);
	if (basePath === headPath) {
		return () => {};
	}
	const baseSource = readFileSync(basePath, "utf8");
	const pinned = pinJourneyClock(baseSource);
	if (pinned === undefined || pinned === baseSource) {
		return () => {};
	}
	writeFileSync(basePath, pinned);
	console.log(
		"benchmark: pinned the base journey's oracle clock so both binaries take the division path",
	);
	return () => {
		writeFileSync(basePath, baseSource);
	};
}

function median(values: number[]): number {
	const sorted = [...values].sort((a, b) => a - b);
	const middle = Math.floor(sorted.length / 2);
	if (sorted.length % 2 === 1) {
		return sorted[middle] ?? 0;
	}
	return Math.round(((sorted[middle - 1] ?? 0) + (sorted[middle] ?? 0)) / 2);
}

function main(): void {
	const { out, buildRoot } = parseArguments();
	mkdirSync(out, { recursive: true });

	// Build the program from `--build-root`. The journey deploys whatever
	// `PINA_SBF_ARTIFACT` points at, which is how the base and head binaries
	// are measured separately.
	run("pina", ["build", "--project", join(buildRoot, PROGRAM_DIR)]);

	const artifact = resolve(buildRoot, `target/deploy/${PROGRAM}.so`);
	const binary = readFileSync(artifact);
	const idl: Idl = JSON.parse(
		readFileSync(resolve(buildRoot, `target/idl/${PROGRAM}.json`), "utf8"),
	) as Idl;
	const names = new Map<number, string>();
	for (const instruction of idl.program?.instructions ?? []) {
		for (const discriminator of instruction.discriminators ?? []) {
			const value = discriminator.constant?.value?.number;
			if (value !== undefined && (discriminator.offset ?? 0) === 0) {
				names.set(value, instruction.name);
			}
		}
	}

	// Three runs give every instruction three samples; the report compares
	// medians, which absorb one-off scheduler noise without hiding a real
	// regression behind averaging.
	const RUNS = 3;
	const samplesPath = resolve(out, "samples.jsonl");
	writeFileSync(samplesPath, "");
	// The base is measured through its own journey harness: the head's test
	// crate can only decode accounts the head's program writes, so a layout
	// change would otherwise break the base benchmark instead of reporting.
	// The clock pin is the exception: it is replayed onto that harness when
	// the base journey does not already contain it.
	const manifest = resolve(buildRoot, TEST_MANIFEST);
	const restoreBaseJourney = alignBaseJourneyClock(buildRoot);
	try {
		for (let attempt = 0; attempt < RUNS; attempt += 1) {
			run(
				"cargo",
				[
					"test",
					"--manifest-path",
					manifest,
					"--locked",
					"--",
					"--ignored",
					JOURNEY_TEST,
				],
				{
					env: {
						PINA_SBF_ARTIFACT: artifact,
						PINA_CU_RECORD_FILE: samplesPath,
					},
				},
			);
		}
	} finally {
		restoreBaseJourney();
	}

	const byDiscriminator = new Map<number, number[]>();
	for (const line of readFileSync(samplesPath, "utf8").split("\n")) {
		if (line.trim() === "") continue;
		const sample = JSON.parse(line) as Sample;
		const bucket = byDiscriminator.get(sample.discriminator) ?? [];
		bucket.push(sample.computeUnits);
		byDiscriminator.set(sample.discriminator, bucket);
	}

	const instructions = [...byDiscriminator.entries()]
		.map(([discriminator, values]) => ({
			name: names.get(discriminator) ?? `discriminator ${discriminator}`,
			discriminator,
			samples: values.length,
			median: median(values),
		}))
		.sort((a, b) => a.discriminator - b.discriminator);

	const benchmark: Benchmark = {
		timestamp: Math.floor(Date.now() / 1000),
		binaryBytes: statSync(artifact).size,
		binarySha256: createHash("sha256").update(binary).digest("hex"),
		instructions,
	};
	writeFileSync(
		resolve(out, "benchmark.json"),
		`${JSON.stringify(benchmark, null, "\t")}\n`,
	);
	console.log(
		`benchmark: ${String(instructions.length)} instructions, ${
			String(benchmark.binaryBytes)
		} byte binary → ${out}`,
	);
}

function invokedDirectly(): boolean {
	const entry = process.argv[1];
	if (entry === undefined) {
		return false;
	}
	return import.meta.url === pathToFileURL(resolve(entry)).href;
}

if (invokedDirectly()) {
	main();
}
