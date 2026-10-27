import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
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
