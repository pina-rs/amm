#!/usr/bin/env node
// Builds the program, measures the deployed binary size, and records the
// compute units of every instruction by running the all-instructions journey
// on a real Surfnet with `PINA_CU_RECORD_FILE` set. The output directory
// holds `benchmark.json`, the single artifact `compare-benchmarks.ts`
// compares between the PR base and head.
//
// Usage: node scripts/benchmark.ts --out <directory>

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const PROGRAM = "pina_amm";
const PROGRAM_DIR = "programs/pina_amm";
const JOURNEY_TEST = "journeys::every_instruction";
const TEST_MANIFEST = "programs/pina_amm/tests/surfpool/Cargo.toml";

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

	// The program is built from `--build-root`, which lets a pull request
	// benchmark the base's binary through this checkout's journey harness:
	// the test deploys whatever `PINA_SBF_ARTIFACT` points at.
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
	for (let attempt = 0; attempt < RUNS; attempt += 1) {
		run(
			"cargo",
			[
				"test",
				"--manifest-path",
				TEST_MANIFEST,
				"--locked",
				"--",
				"--ignored",
				JOURNEY_TEST,
			],
			{
				env: { PINA_SBF_ARTIFACT: artifact, PINA_CU_RECORD_FILE: samplesPath },
			},
		);
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

main();
