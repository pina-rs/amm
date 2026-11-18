import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
	chmodSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { isMap, isScalar, isSeq, parseDocument } from "yaml";

for (const name of ["release-pr", "website"]) {
	test(`${name} only grants release or deployment authority on main pushes`, async () => {
		const source = await readFile(
			new URL(`../.github/workflows/${name}.yml`, import.meta.url),
			"utf8",
		);
		const workflow = parseDocument(source);
		const triggers = workflow.get("on");
		const branches = workflow.getIn(["on", "push", "branches"]);

		assert.deepEqual(workflow.errors, []);
		assert.ok(isMap(triggers));
		assert.deepEqual(
			triggers.items.map(({ key }) => isScalar(key) ? key.value : key),
			["push"],
			"Manual dispatch and other events must not run these privileged workflows",
		);
		assert.ok(isSeq(branches));
		assert.deepEqual(branches.toJSON(), ["main"]);
		assert.equal(workflow.hasIn(["on", "push", "branches-ignore"]), false);
		assert.equal(workflow.hasIn(["on", "push", "tags"]), false);
		assert.equal(workflow.hasIn(["on", "push", "tags-ignore"]), false);
	});
}

test("the clean reproducible build keeps the first digest", async () => {
	const source = await readFile(
		new URL("../.github/workflows/reproducible-build.yml", import.meta.url),
		"utf8",
	);
	const workflow = parseDocument(source);
	assert.deepEqual(workflow.errors, []);
	const steps = workflow.getIn(["jobs", "attest", "steps"]);
	assert.ok(isSeq(steps));
	const second = steps.toJSON().find((step) =>
		step.name === "Build the program (clean second pass)"
	);
	assert.equal(typeof second?.run, "string");
	const script = second.run as string;
	assert.match(script, /rm -rf target/);
	assert.ok(
		script.indexOf("cp -a target/reproducible/.") <
			script.indexOf("rm -rf target"),
		"the first digest must leave target/ before that tree is deleted",
	);

	const root = mkdtempSync(join(tmpdir(), "reproducible-build-"));
	const bin = join(root, "bin");
	mkdirSync(bin);
	writeFileSync(
		join(bin, "devenv"),
		"#!/bin/sh\nmkdir -p target/deploy\nprintf 'second\\n' > target/deploy/pina_amm.so\n",
	);
	chmodSync(join(bin, "devenv"), 0o755);
	mkdirSync(join(root, "target/reproducible"), { recursive: true });
	mkdirSync(join(root, "target/deploy"), { recursive: true });
	writeFileSync(join(root, "target/reproducible/pina_amm-first.so"), "first\n");
	writeFileSync(
		join(root, "target/reproducible/first.sha256"),
		"firsthash  pina_amm-first.so\n",
	);
	writeFileSync(join(root, "target/sentinel"), "from the first build\n");

	try {
		execFileSync("bash", ["-e", "-c", script], {
			cwd: root,
			env: { ...process.env, PATH: `${bin}:${process.env.PATH ?? ""}` },
		});
		assert.equal(
			readFileSync(join(root, "target/reproducible/first.sha256"), "utf8"),
			"firsthash  pina_amm-first.so\n",
		);
		assert.equal(
			readFileSync(join(root, "target/reproducible/pina_amm-first.so"), "utf8"),
			"first\n",
		);
		assert.equal(
			readFileSync(
				join(root, "target/reproducible/pina_amm-second.so"),
				"utf8",
			),
			"second\n",
		);
		assert.match(
			readFileSync(join(root, "target/reproducible/second.sha256"), "utf8"),
			/pina_amm-second\.so/,
		);
		assert.equal(existsSync(join(root, "target/sentinel")), false);
	} finally {
		rmSync(root, { recursive: true, force: true });
	}
});
