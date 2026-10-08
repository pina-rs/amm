import { describe, expect, it } from "vitest";
import {
	docIdForPath,
	editUrlFor,
	extractDescription,
	extractTitle,
	guideTitleForBareLink,
	isRepoDoc,
	type RepoDocsConfig,
	rewriteRepoLink,
} from "./repo-docs";

const config: RepoDocsConfig = {
	repoRoot: "/repo",
	docsDir: "/repo/docs",
	route: "docs",
	repoUrl: "https://github.com/pina-rs/amm",
	branch: "main",
};

describe("docIdForPath", () => {
	it("serves guides under the route and readme files as indexes", () => {
		expect(docIdForPath("math.md", "docs")).toBe("docs/math");
		expect(docIdForPath("readme.md", "docs")).toBe("docs");
		expect(docIdForPath("guides/README.md", "docs")).toBe("docs/guides");
		expect(docIdForPath("guides/swap.md", "docs")).toBe("docs/guides/swap");
	});
});

describe("extractTitle", () => {
	it("returns the first level-one heading as plain text", () => {
		expect(extractTitle("# The `pina-amm` CLI\n\nBody")).toBe(
			"The pina-amm CLI",
		);
		expect(extractTitle("---\ntitle: x\n---\n## Not this\n# Fees #\n")).toBe(
			"Fees",
		);
		expect(extractTitle("No heading here")).toBeUndefined();
	});
});

describe("extractDescription", () => {
	it("returns the first prose paragraph, skipping structure", () => {
		const markdown = [
			"# Fees",
			"",
			"| a | b |",
			"| - | - |",
			"",
			"```text",
			"fee = ceil(a * t / D)",
			"",
			"still code",
			"```",
			"",
			"- a list item",
			"  that continues",
			"",
			"Every swap pays a **trade fee**, part of which",
			"can go to the [protocol](fees.md).",
		].join("\n");

		expect(extractDescription(markdown)).toBe(
			"Every swap pays a trade fee, part of which can go to the protocol.",
		);
	});

	it("clips long paragraphs at a word boundary", () => {
		const description = extractDescription(`# T\n\n${"word ".repeat(60)}`);

		expect(description?.length).toBeLessThanOrEqual(160);
		expect(description?.endsWith("word…")).toBe(true);
	});
});

describe("rewriteRepoLink", () => {
	const from = "/repo/docs/fees.md";

	it("routes links between guides and keeps the fragment", () => {
		expect(rewriteRepoLink("math.md", from, config)).toBe("/docs/math/");
		expect(rewriteRepoLink("architecture.md#reserves", from, config)).toBe(
			"/docs/architecture/#reserves",
		);
		expect(rewriteRepoLink("readme.md", from, config)).toBe("/docs/");
	});

	it("sends other repository paths to GitHub", () => {
		expect(rewriteRepoLink("../SECURITY.md", from, config)).toBe(
			"https://github.com/pina-rs/amm/blob/main/SECURITY.md",
		);
		expect(
			rewriteRepoLink("../programs/pina_amm/src/math.rs#L10", from, config),
		).toBe(
			"https://github.com/pina-rs/amm/blob/main/programs/pina_amm/src/math.rs#L10",
		);
	});

	it("leaves absolute, root-relative, and fragment links alone", () => {
		for (
			const url of [
				"https://solana.com",
				"mailto:security@pina.rs",
				"/docs/",
				"#reserves",
				"//cdn.example",
			]
		) {
			expect(rewriteRepoLink(url, from, config)).toBe(url);
		}
	});

	it("leaves paths that escape the repository alone", () => {
		expect(rewriteRepoLink("../../elsewhere.md", from, config)).toBe(
			"../../elsewhere.md",
		);
	});
});

describe("editUrlFor and isRepoDoc", () => {
	it("points edits at the file on the branch", () => {
		expect(editUrlFor("/repo/docs/math.md", config)).toBe(
			"https://github.com/pina-rs/amm/edit/main/docs/math.md",
		);
	});

	it("recognises guides only", () => {
		expect(isRepoDoc("/repo/docs/math.md", config)).toBe(true);
		expect(isRepoDoc("/repo/README.md", config)).toBe(false);
		expect(isRepoDoc("/repo/docs/diagram.svg", config)).toBe(false);
	});
});

describe("guideTitleForBareLink", () => {
	const from = "/repo/docs/readme.md";
	const guides: Record<string, string> = { "/repo/docs/math.md": "# Math\n" };
	const read = (path: string) => guides[path];

	it("names a bare link to a guide after the guide", () => {
		expect(guideTitleForBareLink("math.md", "math.md", from, config, read))
			.toBe("Math");
		expect(
			guideTitleForBareLink("math.md", "math.md#rounding", from, config, read),
		).toBe("Math");
	});

	it("keeps written link text, other files, and missing guides", () => {
		expect(guideTitleForBareLink("the math", "math.md", from, config, read))
			.toBeUndefined();
		expect(
			guideTitleForBareLink("../README.md", "../README.md", from, config, read),
		).toBeUndefined();
		expect(guideTitleForBareLink("gone.md", "gone.md", from, config, read))
			.toBeUndefined();
	});
});
