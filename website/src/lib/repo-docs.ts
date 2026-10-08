/**
 * Helpers that let the repository's `docs/*.md` guides stay plain GitHub
 * Markdown while the site renders them as Starlight pages.
 *
 * A guide's first `# Heading` becomes the page title, its first paragraph the
 * page description, and its relative links are rewritten to site routes (for
 * other guides) or GitHub URLs (for anything else in the repository).
 */
import { existsSync, statSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";

export interface RepoDocsConfig {
	/** Absolute path of the repository checkout. */
	repoRoot: string;
	/** Absolute path of the directory holding the guides. */
	docsDir: string;
	/** Route segment the guides are served under, without slashes. */
	route: string;
	/** GitHub repository URL, without a trailing slash. */
	repoUrl: string;
	/** Branch that edit and source links point at. */
	branch: string;
}

const DESCRIPTION_LIMIT = 160;
const NON_PROSE_BLOCK = /^\s*(?:#|\||[-*+]\s|\d+\.\s|>|<)/;
const FENCE = /^\s*(?:```|~~~)/;

/**
 * The content-collection id (and therefore the URL path) of a guide. A
 * directory's `readme.md` is that directory's index page.
 */
export function docIdForPath(relativePath: string, route: string): string {
	const segments = relativePath.split(sep).join("/").replace(/\.md$/i, "")
		.split("/");
	if (segments.at(-1)?.toLowerCase() === "readme") {
		segments.pop();
	}
	return [route, ...segments].join("/");
}

/** Turns inline Markdown into plain text for titles and descriptions. */
export function toPlainText(markdown: string): string {
	return markdown
		.replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
		.replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
		.replace(/`([^`]+)`/g, "$1")
		.replace(/(\*\*|__)(.+?)\1/g, "$2")
		.replace(/(\*|_)(.+?)\1/g, "$2")
		.replace(/\s+/g, " ")
		.trim();
}

function stripFrontmatter(markdown: string): string {
	return markdown.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, "");
}

/** The text of the first level-one heading, or `undefined` when there is none. */
export function extractTitle(markdown: string): string | undefined {
	for (const line of stripFrontmatter(markdown).split(/\r?\n/)) {
		const heading = /^#\s+(.+?)\s*#*\s*$/.exec(line);
		if (heading?.[1]) {
			return toPlainText(heading[1]);
		}
	}
	return undefined;
}

/**
 * The first prose paragraph after the title, as plain text clipped to a
 * search-snippet length. Tables, lists, code, quotes, and HTML are skipped.
 */
export function extractDescription(markdown: string): string | undefined {
	let inFence = false;
	let skippingBlock = false;
	const paragraph: string[] = [];
	// The trailing blank line closes a paragraph that ends the file.
	for (const line of [...stripFrontmatter(markdown).split(/\r?\n/), ""]) {
		if (FENCE.test(line)) {
			inFence = !inFence;
			continue;
		}
		if (inFence) {
			continue;
		}
		if (line.trim() === "") {
			if (paragraph.length > 0) {
				return clipDescription(toPlainText(paragraph.join(" ")));
			}
			skippingBlock = false;
			continue;
		}
		if (skippingBlock) {
			continue;
		}
		if (paragraph.length === 0 && NON_PROSE_BLOCK.test(line)) {
			skippingBlock = true;
			continue;
		}
		paragraph.push(line.trim());
	}
	return undefined;
}

function clipDescription(text: string): string {
	if (text.length <= DESCRIPTION_LIMIT) {
		return text;
	}
	const clipped = text.slice(0, DESCRIPTION_LIMIT - 1);
	return `${clipped.slice(0, clipped.lastIndexOf(" "))}…`;
}

/** The GitHub URL that edits a file in the repository. */
export function editUrlFor(filePath: string, config: RepoDocsConfig): string {
	const repoPath = relative(config.repoRoot, filePath).split(sep).join("/");
	return `${config.repoUrl}/edit/${config.branch}/${repoPath}`;
}

const EXTERNAL_URL = /^(?:[a-z][a-z\d+.-]*:|\/\/)/i;

/**
 * Rewrites a link found in a guide at `fromFile`.
 *
 * - A link to another guide becomes its site route, keeping the fragment.
 * - A link to any other repository path becomes a GitHub URL.
 * - Absolute URLs, root-relative paths, and bare fragments are left alone.
 */
export function rewriteRepoLink(
	url: string,
	fromFile: string,
	config: RepoDocsConfig,
): string {
	if (
		url === "" || url.startsWith("#") || url.startsWith("/") ||
		EXTERNAL_URL.test(url)
	) {
		return url;
	}
	const hashIndex = url.indexOf("#");
	const path = hashIndex === -1 ? url : url.slice(0, hashIndex);
	const fragment = hashIndex === -1 ? "" : url.slice(hashIndex);
	const target = resolve(dirname(fromFile), decodeURI(path));

	const fromDocs = relative(config.docsDir, target);
	const insideDocs = fromDocs !== "" && !fromDocs.startsWith("..") &&
		!isAbsolute(fromDocs);
	if (insideDocs && /\.md$/i.test(target)) {
		return `/${docIdForPath(fromDocs, config.route)}/${fragment}`;
	}

	const fromRoot = relative(config.repoRoot, target).split(sep).join("/");
	if (fromRoot.startsWith("..") || isAbsolute(fromRoot)) {
		return url;
	}
	const kind = existsSync(target) && statSync(target).isDirectory()
		? "tree"
		: "blob";
	return `${config.repoUrl}/${kind}/${config.branch}/${fromRoot}${fragment}`;
}

/**
 * The title of the guide a link points at, when the link's text is just the
 * linked file (`[math.md](math.md#rounding)`, natural on GitHub) and the
 * target is a guide. `undefined` leaves the text alone.
 */
export function guideTitleForBareLink(
	text: string,
	url: string,
	fromFile: string,
	config: RepoDocsConfig,
	readGuide: (path: string) => string | undefined,
): string | undefined {
	const path = url.split("#")[0] ?? "";
	if (
		path === "" || (text !== url && text !== path) || EXTERNAL_URL.test(url) ||
		url.startsWith("/")
	) {
		return undefined;
	}
	const target = resolve(dirname(fromFile), decodeURI(path));
	if (!isRepoDoc(target, config)) {
		return undefined;
	}
	const markdown = readGuide(target);
	return markdown === undefined ? undefined : extractTitle(markdown);
}

/** Whether a rendered file is one of the guides. */
export function isRepoDoc(filePath: string, config: RepoDocsConfig): boolean {
	const fromDocs = relative(config.docsDir, filePath);
	return !fromDocs.startsWith("..") && !isAbsolute(fromDocs) &&
		/\.md$/i.test(filePath);
}
