# amm.pina.rs

The Pina AMM website: a landing page and the documentation, built with [Astro](https://astro.build) and [Starlight](https://starlight.astro.build) and served by a [Cloudflare Worker](https://developers.cloudflare.com/workers/).

## Develop

```sh
devenv shell dev:website      # http://localhost:4321
devenv shell build:website    # static build into website/dist
pnpm --dir website preview    # build, then serve through the Worker with wrangler dev
pnpm --dir website check      # astro check and the Worker's type check
pnpm --dir website test       # unit tests
```

`verify:all` runs the check, the tests, and the build, so CI fails on a broken page or a guide the site cannot render.

## How it fits together

| Path                    | What it is                                                                           |
| ----------------------- | ------------------------------------------------------------------------------------ |
| `src/pages/index.astro` | The landing page                                                                     |
| `src/components/`       | The live pool, code tabs, and page shell                                             |
| `src/lib/playground.ts` | The live pool's model, priced by the program's exact-input formula in `docs/math.md` |
| `src/lib/repo-docs*.ts` | Renders the repository's `docs/*.md` as the `/docs/` pages                           |
| `src/styles/tokens.css` | Design tokens shared by the landing page and the docs                                |
| `worker/index.ts`       | The Worker: API routes under `/api/*`; everything else is a static asset             |
| `wrangler.jsonc`        | Worker configuration, including the `amm.pina.rs` custom domain                      |

### Documentation

The guides in [`docs/`](../docs) stay plain GitHub Markdown and remain the only copy. The site reads them directly:

- a guide's first `# Heading` is its page title and its first paragraph its description;
- links between guides become site routes, and links to anything else in the repository become GitHub URLs;
- a bare link such as `[math.md](math.md)` shows the linked guide's title.

Add a guide by adding a Markdown file to `docs/` and listing it in the sidebar in `astro.config.ts`.

### API routes

Static pages never reach the Worker: `wrangler.jsonc` runs it first only for `/api/*`. Add an endpoint to the `routes` table in `worker/index.ts` and a test beside it in `worker/index.test.ts`. `GET /api/health` is the only route today.

## Deploy

Every push to `main` that touches the site or the docs runs [`website.yml`](../.github/workflows/website.yml), which builds the site and runs `wrangler deploy`. To deploy by hand, export the two variables below and run `devenv shell deploy:website`.

One-time setup, done by an owner of the Cloudflare account:

1. Add the `pina.rs` zone to Cloudflare and point the domain's nameservers at it at the registrar. The Worker serves `amm.pina.rs` as a [custom domain](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/), which needs the zone in the same account.
2. Create an API token from the **Edit Cloudflare Workers** template, limited to that account and the `pina.rs` zone.
3. In the repository's settings, create a `website` environment. Add the account ID as the repository variable `CLOUDFLARE_ACCOUNT_ID` and the token as the environment secret `CLOUDFLARE_API_TOKEN`.

The deploy job is skipped until `CLOUDFLARE_ACCOUNT_ID` is set. Cloudflare's free plan covers the site: static assets are free, and the Worker only runs for API requests.
