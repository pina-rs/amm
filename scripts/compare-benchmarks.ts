#!/usr/bin/env node
// Compares two `benchmark.json` artifacts produced by `benchmark.ts` against
// the policy in `benchmark-policy.json`, writes a markdown report, and exits
// nonzero when the policy is violated so the performance workflow can gate
// the pull request.
//
// Usage: node scripts/compare-benchmarks.ts --base <dir> --head <dir>
//          [--policy scripts/benchmark-policy.json]
//          --markdown report.md --json report.json

import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

interface Policy {
	binarySize?: { maxIncreaseBytes?: number };
	computeUnits?: { thresholdPercent?: number };
}

interface BenchmarkInstruction {
	name: string;
	discriminator: number;
	samples: number;
	median: number;
}

interface Benchmark {
	binaryBytes: number;
	instructions: BenchmarkInstruction[];
}

interface Row {
	name: string;
	base?: number;
	head?: number;
	deltaPercent?: number;
	status: "ok" | "improved" | "worse" | "missing" | "new";
}

function parseArguments(): {
	base: string;
	head: string;
	policy: string;
	markdown: string;
	json: string;
} {
	const value = (flag: string, fallback?: string): string => {
		const index = process.argv.indexOf(flag);
		const raw = index === -1 ? undefined : process.argv[index + 1];
		if (raw === undefined) {
			if (fallback === undefined) {
				throw new Error(`missing ${flag}`);
			}
			return resolve(fallback);
		}
		return resolve(raw);
	};
	return {
		base: value("--base"),
		head: value("--head"),
		policy: value("--policy", "scripts/benchmark-policy.json"),
		markdown: value("--markdown"),
		json: value("--json"),
	};
}

function deltaPercent(base: number, head: number): number {
	if (base === 0) {
		return head === 0 ? 0 : Number.POSITIVE_INFINITY;
	}
	return ((head - base) / base) * 100;
}

function formatPercent(value: number): string {
	if (!Number.isFinite(value)) {
		return "new";
	}
	const sign = value > 0 ? "+" : "";
	return `${sign}${value.toFixed(2)}%`;
}

function main(): void {
	const args = parseArguments();
	const policy = JSON.parse(readFileSync(args.policy, "utf8")) as Policy;
	const threshold = policy.computeUnits?.thresholdPercent ?? 0;
	const maxIncreaseBytes = policy.binarySize?.maxIncreaseBytes ?? 0;
	const base = JSON.parse(
		readFileSync(resolve(args.base, "benchmark.json"), "utf8"),
	) as Benchmark;
	const head = JSON.parse(
		readFileSync(resolve(args.head, "benchmark.json"), "utf8"),
	) as Benchmark;

	const rows: Row[] = [];
	const baseInstructions = new Map(
		base.instructions.map((i) => [i.discriminator, i]),
	);
	const headInstructions = new Map(
		head.instructions.map((i) => [i.discriminator, i]),
	);
	for (const [discriminator, baseInstruction] of baseInstructions) {
		const headInstruction = headInstructions.get(discriminator);
		if (headInstruction === undefined) {
			rows.push({
				name: baseInstruction.name,
				base: baseInstruction.median,
				status: "missing",
			});
			continue;
		}
		const change = deltaPercent(baseInstruction.median, headInstruction.median);
		rows.push({
			name: baseInstruction.name,
			base: baseInstruction.median,
			head: headInstruction.median,
			deltaPercent: change,
			status: change > threshold ? "worse" : change < 0 ? "improved" : "ok",
		});
	}
	for (const [discriminator, headInstruction] of headInstructions) {
		if (!baseInstructions.has(discriminator)) {
			rows.push({
				name: headInstruction.name,
				head: headInstruction.median,
				status: "new",
			});
		}
	}

	const sizeDelta = head.binaryBytes - base.binaryBytes;
	const sizeViolated = sizeDelta > maxIncreaseBytes;
	const violations = rows.filter((row) =>
		row.status === "worse" || row.status === "missing"
	);

	const lines: string[] = [
		"## Performance",
		"",
		`Deployed binary: **${String(base.binaryBytes)} → ${
			String(head.binaryBytes)
		} bytes** (${
			formatPercent(deltaPercent(base.binaryBytes, head.binaryBytes))
		})`,
		"",
		"| Instruction | Base CU | Head CU | Change | Status |",
		"| --- | --- | --- | --- | --- |",
	];
	for (const row of rows) {
		const status = row.status === "worse"
			? `**regression (> ${threshold.toString()}%)**`
			: row.status === "missing"
			? "**missing measurement**"
			: row.status === "new"
			? "new baseline"
			: row.status === "improved"
			? "improved"
			: "ok";
		lines.push(
			`| ${row.name} | ${row.base === undefined ? "—" : String(row.base)} | ${
				row.head === undefined ? "—" : String(row.head)
			} | ${
				row.deltaPercent === undefined ? "—" : formatPercent(row.deltaPercent)
			} | ${status} |`,
		);
	}
	lines.push("");
	if (sizeViolated) {
		lines.push(
			`- 🚩 The deployed binary grew by ${
				String(sizeDelta)
			} bytes; the policy allows at most ${String(maxIncreaseBytes)}.`,
		);
	}
	if (violations.length > 0) {
		lines.push(
			`- 🚩 ${
				String(violations.length)
			} instruction(s) regressed beyond the ${threshold.toString()}% threshold or lost their measurement.`,
		);
	}
	if (!sizeViolated && violations.length === 0) {
		lines.push(
			"- ✅ No compute-unit regression beyond the threshold and no binary-size increase.",
		);
	}
	lines.push("");
	lines.push(
		"Measured on an offline Surfnet by `scripts/benchmark.ts`; the policy lives in `scripts/benchmark-policy.json`. A regression blocks auto-merge until the pull request carries the `performance-approved` label.",
	);
	const markdown = lines.join("\n");
	writeFileSync(args.markdown, `${markdown}\n`);
	writeFileSync(
		args.json,
		`${
			JSON.stringify(
				{
					violated: sizeViolated || violations.length > 0,
					binaryBytes: { base: base.binaryBytes, head: head.binaryBytes },
					rows,
				},
				null,
				"\t",
			)
		}\n`,
	);
	console.log(markdown);
	process.exitCode = sizeViolated || violations.length > 0 ? 1 : 0;
}

main();
