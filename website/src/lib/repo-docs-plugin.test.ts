import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { markdownToHtml } from "satteri";
import { afterAll, describe, expect, it } from "vitest";
import type { RepoDocsConfig } from "./repo-docs";
import { repoDocsPlugin } from "./repo-docs-plugin";

const config: RepoDocsConfig = {
	repoRoot: "/repo",
	docsDir: "/repo/docs",
	route: "docs",
	repoUrl: "https://github.com/pina-rs/amm",
	branch: "main",
};

// `markdownToHtml` is synchronous unless a plugin is async; awaiting covers both.
async function render(
	markdown: string,
	filePath: string,
	options: RepoDocsConfig = config,
): Promise<string> {
	const result = await markdownToHtml(markdown, {
		mdastPlugins: [repoDocsPlugin(options)],
		fileURL: pathToFileURL(filePath),
	});
	return result.html;
}

describe("repoDocsPlugin", () => {
	it("drops the leading title and rewrites links in a guide", async () => {
		const html = await render(
			"# Fees\n\nSee [math](math.md#rounding) and [policy][p].\n\n[p]: ../SECURITY.md\n",
			"/repo/docs/fees.md",
		);

		expect(html).not.toContain("<h1");
		expect(html).toContain('href="/docs/math/#rounding"');
		expect(html).toContain(
			'href="https://github.com/pina-rs/amm/blob/main/SECURITY.md"',
		);
	});

	it("keeps later level-one headings", async () => {
		const html = await render(
			"Intro\n\n# Not the title\n",
			"/repo/docs/fees.md",
		);

		expect(html).toContain("<h1");
	});

	it("leaves Markdown outside the guides untouched", async () => {
		const html = await render(
			"# Notes\n\n[math](math.md)\n",
			"/repo/release-notes/v0.1.0.md",
		);

		expect(html).toContain("<h1");
		expect(html).toContain('href="math.md"');
	});
});

describe("repoDocsPlugin on real files", () => {
	const repoRoot = mkdtempSync(join(tmpdir(), "repo-docs-"));
	mkdirSync(join(repoRoot, "docs"));
	writeFileSync(
		join(repoRoot, "docs", "math.md"),
		"# Math\n\nRounding rules.\n",
	);
	const realConfig: RepoDocsConfig = {
		...config,
		repoRoot,
		docsDir: join(repoRoot, "docs"),
	};
	afterAll(() => rmSync(repoRoot, { recursive: true, force: true }));

	it("titles bare links to guides", async () => {
		const html = await render(
			"# Guides\n\n[math.md](math.md) and [the rules](math.md)\n",
			join(repoRoot, "docs", "readme.md"),
			realConfig,
		);

		expect(html).toContain('<a href="/docs/math/">Math</a>');
		expect(html).toContain('<a href="/docs/math/">the rules</a>');
	});
});
