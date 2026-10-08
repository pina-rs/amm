import { glob, type Loader, type LoaderContext } from "astro/loaders";
import { readFileSync } from "node:fs";
import { relative } from "node:path";
import { pathToFileURL } from "node:url";
import {
	docIdForPath,
	editUrlFor,
	extractDescription,
	extractTitle,
} from "./repo-docs";
import { resolveRepoDocsConfig } from "./site";

/**
 * Loads the repository's `docs/*.md` guides into Starlight's `docs`
 * collection without asking them to carry frontmatter: the title, description,
 * and edit link come from the file itself. Frontmatter, when present, wins.
 */
export function repoDocsLoader(): Loader {
	return {
		name: "repo-docs-loader",
		load: (context) => {
			const config = resolveRepoDocsConfig(context.config.root);
			const guides = glob({
				base: pathToFileURL(config.docsDir),
				pattern: "**/*.md",
				generateId: ({ entry }) => docIdForPath(entry, config.route),
			});
			const parseData: LoaderContext["parseData"] = (entry) => {
				const { filePath } = entry;
				if (filePath === undefined) {
					throw new Error(`Guide "${entry.id}" has no file path`);
				}
				const markdown = readFileSync(filePath, "utf8");
				const title = extractTitle(markdown);
				if (title === undefined && !("title" in entry.data)) {
					throw new Error(
						`${relative(config.repoRoot, filePath)} needs a "# Title" heading`,
					);
				}
				return context.parseData({
					...entry,
					data: {
						title,
						description: extractDescription(markdown),
						editUrl: editUrlFor(filePath, config),
						...entry.data,
					},
				});
			};
			return guides.load({ ...context, parseData });
		},
	};
}
