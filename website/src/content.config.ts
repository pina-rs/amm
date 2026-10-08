import { docsSchema } from "@astrojs/starlight/schema";
import { defineCollection } from "astro:content";
import { repoDocsLoader } from "./lib/repo-docs-loader";

export const collections = {
	docs: defineCollection({ loader: repoDocsLoader(), schema: docsSchema() }),
};
