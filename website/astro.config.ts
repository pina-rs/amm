import { satteri } from "@astrojs/markdown-satteri";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { repoDocsPlugin } from "./src/lib/repo-docs-plugin";
import { resolveRepoDocsConfig, site } from "./src/lib/site";

const repoDocs = resolveRepoDocsConfig(new URL("./", import.meta.url));

export default defineConfig({
	site: site.url,
	markdown: {
		processor: satteri({ mdastPlugins: [repoDocsPlugin(repoDocs)] }),
	},
	integrations: [
		starlight({
			title: site.name,
			// The landing page design renders the not-found page instead.
			disable404Route: true,
			description: site.description,
			logo: { src: "./src/assets/mark.svg" },
			favicon: "/favicon.svg",
			social: [{ icon: "github", label: "GitHub", href: site.repository }],
			customCss: ["./src/styles/tokens.css", "./src/styles/starlight.css"],
			expressiveCode: { themes: ["github-dark", "github-light"] },
			sidebar: [
				{
					label: "Start here",
					items: [
						{ label: "Overview", slug: "docs" },
						"docs/integrating",
						"docs/architecture",
					],
				},
				{
					label: "How it works",
					items: [
						"docs/math",
						"docs/fees",
						"docs/security",
						"docs/performance",
					],
				},
				{
					label: "Clients",
					items: [
						"docs/typescript",
						"docs/rust-client",
						"docs/dart",
						"docs/cpi",
						"docs/cli",
					],
				},
				{ label: "Reference", items: ["docs/instructions"] },
				{ label: "Operating", items: ["docs/deploying", "docs/releasing"] },
			],
		}),
	],
});
