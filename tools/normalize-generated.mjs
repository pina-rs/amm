// Post-generation fixes for renderer output that does not compile as emitted.
// Runs inside `generate:clients`; generated files are never patched by hand.
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Codama's Dart renderer emits `Object.hash(...)` with one argument per field.
 * Dart caps `Object.hash` at 20 arguments, so accounts with more fields (such as
 * `LaunchConfig`) fail to compile. `Object.hashAll` takes the same fields as a
 * list and produces an equally good hash.
 */
export function normalizeDartHashes(source) {
	return source.replace(/Object\.hash\(([\w\s,]+)\)/g, (match, fields) => {
		const count = fields.split(",").filter((field) => field.trim()).length;
		return count > 20 ? `Object.hashAll([${fields}])` : match;
	});
}

function dartFiles(directory) {
	return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
		const path = join(directory, entry.name);
		if (entry.isDirectory()) return dartFiles(path);
		return entry.name.endsWith(".dart") ? [path] : [];
	});
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
	const root = resolve(process.argv[2] ?? "clients/dart/lib");
	for (const file of dartFiles(root)) {
		const source = readFileSync(file, "utf8");
		const normalized = normalizeDartHashes(source);
		if (normalized !== source) writeFileSync(file, normalized);
	}
}
