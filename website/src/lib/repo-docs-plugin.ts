import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineMdastPlugin, type MdastPluginEntry } from "satteri";
import {
	guideTitleForBareLink,
	isRepoDoc,
	type RepoDocsConfig,
	rewriteRepoLink,
} from "./repo-docs";

function readGuide(path: string): string | undefined {
	return existsSync(path) ? readFileSync(path, "utf8") : undefined;
}

/**
 * A Sätteri plugin that adapts a repository guide for the site: it drops the
 * leading `# Title` (the page header renders it from the page data), rewrites
 * relative links, and replaces a bare `[math.md](math.md)` link's text with
 * the guide's title. Markdown outside the guides directory is left alone,
 * because the factory leaves the plugin out for it.
 *
 * Astro caches each guide's render by its own content, so after renaming a
 * guide, bare links to it in unchanged guides refresh on a clean build.
 */
export function repoDocsPlugin(config: RepoDocsConfig): MdastPluginEntry {
	return ({ fileURL }) => {
		if (fileURL === undefined) {
			return null;
		}
		const filePath = fileURLToPath(fileURL);
		if (!isRepoDoc(filePath, config)) {
			return null;
		}
		return defineMdastPlugin({
			name: "pina-repo-docs",
			heading(node, context) {
				if (
					node.depth === 1 && context.parent(node)?.type === "root" &&
					context.indexOf(node) === 0
				) {
					context.removeNode(node);
				}
			},
			link(node, context) {
				const title = guideTitleForBareLink(
					context.textContent(node),
					node.url,
					filePath,
					config,
					readGuide,
				);
				if (title !== undefined) {
					context.setProperty(node, "children", [{
						type: "text",
						value: title,
					}]);
				}
				context.setProperty(
					node,
					"url",
					rewriteRepoLink(node.url, filePath, config),
				);
			},
			definition(node, context) {
				context.setProperty(
					node,
					"url",
					rewriteRepoLink(node.url, filePath, config),
				);
			},
		});
	};
}
