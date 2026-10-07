import { defineConfig } from "tsup";

// One bundled ESM and CommonJS build with a single declaration file, so the
// generated sources' extensionless relative imports never reach consumers.
export default defineConfig({
	entry: ["src/index.ts"],
	format: ["esm", "cjs"],
	dts: true,
	clean: true,
	sourcemap: true,
	treeshake: true,
	target: "es2022",
});
