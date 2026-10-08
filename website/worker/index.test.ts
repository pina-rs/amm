import { describe, expect, it } from "vitest";
import worker, { type Env } from "./index";

const assetResponse = new Response("<!doctype html>", {
	headers: { "Content-Type": "text/html" },
});
const env: Env = {
	ASSETS: {
		fetch: async () => assetResponse.clone(),
		connect: () => {
			throw new Error("not used");
		},
	},
};
function call(path: string, init?: RequestInit): Promise<Response> {
	return worker.fetch(new Request(`https://amm.pina.rs${path}`, init), env);
}

describe("worker", () => {
	it("reports health", async () => {
		const response = await call("/api/health");

		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({ status: "ok" });
		expect(response.headers.get("Cache-Control")).toBe("no-store");
	});

	it("rejects unsupported methods with the allowed list", async () => {
		const response = await call("/api/health", { method: "DELETE" });

		expect(response.status).toBe(405);
		expect(response.headers.get("Allow")).toBe("GET");
	});

	it("answers unknown API paths with JSON 404", async () => {
		const response = await call("/api/nope");

		expect(response.status).toBe(404);
		expect(await response.json()).toEqual({ error: "not_found" });
	});

	it("serves everything else from the static assets", async () => {
		const response = await call("/docs/");

		expect(response.status).toBe(200);
		expect(await response.text()).toBe("<!doctype html>");
	});
});
