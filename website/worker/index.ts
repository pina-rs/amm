/**
 * The Cloudflare Worker behind amm.pina.rs.
 *
 * Static pages are served straight from the built assets and never reach
 * this code: `wrangler.jsonc` only runs the Worker first for `/api/*`. Add an
 * endpoint by adding a route below.
 */

export interface Env {
	ASSETS: Fetcher;
}

type Handler = (request: Request, env: Env) => Response | Promise<Response>;

const routes: Record<string, Partial<Record<"GET" | "POST", Handler>>> = {
	"/api/health": {
		GET: () =>
			Response.json({ status: "ok" }, {
				headers: { "Cache-Control": "no-store" },
			}),
	},
};

function isMethod(method: string): method is "GET" | "POST" {
	return method === "GET" || method === "POST";
}

export default {
	async fetch(request: Request, env: Env): Promise<Response> {
		const { pathname } = new URL(request.url);
		const route = routes[pathname];
		if (route === undefined) {
			return pathname.startsWith("/api/")
				? Response.json({ error: "not_found" }, { status: 404 })
				: env.ASSETS.fetch(request);
		}
		const handler = isMethod(request.method)
			? route[request.method]
			: undefined;
		if (handler === undefined) {
			return Response.json(
				{ error: "method_not_allowed" },
				{ status: 405, headers: { Allow: Object.keys(route).join(", ") } },
			);
		}
		return handler(request, env);
	},
} satisfies ExportedHandler<Env>;
