import { fileURLToPath } from "node:url";
import type { RepoDocsConfig } from "./repo-docs";

/** Facts about the project that the landing page and docs share. */
export const site = {
	name: "Pina AMM",
	url: "https://amm.pina.rs",
	description:
		"A permissionless constant-product market maker for Solana, built with Pina. Fees lock when a pool is created, and a swap costs about 4,700 compute units.",
	repository: "https://github.com/pina-rs/amm",
	branch: "main",
	programId: "pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV",
	pina: "https://github.com/pina-rs/pina",
	bondingCurve: "https://bonding-curve.pina.rs",
} as const;

/** The published packages, in the order the landing page lists them. */
export const packages = [
	{
		name: "@pina-rs/amm",
		registry: "npm",
		href: "https://www.npmjs.com/package/@pina-rs/amm",
		summary: "TypeScript client for @solana/kit",
	},
	{
		name: "pina_amm_client",
		registry: "crates.io",
		href: "https://crates.io/crates/pina_amm_client",
		summary: "Rust client: builders, decoders, PDAs, events",
	},
	{
		name: "pina_amm_cpi",
		registry: "crates.io",
		href: "https://crates.io/crates/pina_amm_cpi",
		summary: "no_std client for cross-program calls",
	},
	{
		name: "pina_amm",
		registry: "pub.dev",
		href: "https://pub.dev/packages/pina_amm",
		summary: "Dart and Flutter client for solana_kit",
	},
	{
		name: "pina_amm_cli",
		registry: "crates.io",
		href: "https://crates.io/crates/pina_amm_cli",
		summary: "The pina-amm command-line tool",
	},
] as const;

/** Where the guides live relative to the site, and how they are served. */
const docsSource = { directory: "../docs/", route: "docs" } as const;

/** Resolves the guide configuration against the site's root directory. */
export function resolveRepoDocsConfig(siteRoot: URL): RepoDocsConfig {
	return {
		repoRoot: fileURLToPath(new URL("../", siteRoot)),
		docsDir: fileURLToPath(new URL(docsSource.directory, siteRoot)),
		route: docsSource.route,
		repoUrl: site.repository,
		branch: site.branch,
	};
}
