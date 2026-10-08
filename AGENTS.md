# Repository agent instructions

## Pre-release protocol policy

This repository is pre-release. The program has no release tag and no supported deployment.

- Breaking changes are allowed while the protocol is designed. Use canonical, unversioned names; do not keep compatibility aliases for unreleased designs.
- This policy ends once a `v*` tag exists **and** the program is live as a supported deployment. After that, incompatible changes need an explicit compatibility plan approved by the maintainer.

## Program changes

- Run commands through `devenv shell <script>`; see `devenv.nix` for the full list.
- After changing any account, instruction, or event, run `pina migrations create --project programs/pina_amm`, then `devenv shell generate:clients`, and commit `migrations/` with the regenerated clients.
- Never edit files under `clients/**/generated/`, `clients/dart/lib/src/generated/`, `.pina-generated.json`, or `migrations/` by hand. Client manifests and entrypoints (`Cargo.toml`, `package.json`, `pubspec.yaml`, `src/lib.rs`, `src/index.ts`) are hand-owned.
- Every hand-written public item and field carries a doc comment. Error variants keep a one-line doc comment: generated clients use that line as the error message.
- Doc comments on accounts, instructions, and events flow into every generated client. Use plain code spans there, never rustdoc intra-doc links.
- Keep arithmetic checked and rounded in the pool's favour. Add a property test for any new formula.
- Any change to compute cost must keep `compute_units_stay_within_budget` passing; raise a ceiling only with a measured reason.

## Verification before a pull request

```sh
devenv shell lint:all        # Pina security lints, clippy, rustdoc, dprint, tsc, dart analyze, monochange
devenv shell check:clients   # generated clients match the program
devenv shell test:unit
devenv shell test:surfpool   # real SBF artifact on an offline Surfnet, including the CLI
```

## Website

`website/` is amm.pina.rs: an Astro and Starlight site served by a Cloudflare Worker. It renders `docs/*.md` directly, so the guides stay plain GitHub Markdown with a `# Title` first line and relative links; never copy a guide into the site. The landing page's live pool uses the exact-input formula from `docs/math.md`; keep `website/src/lib/constant-product.ts` in step with `programs/pina_amm/src/math.rs`. The deploy job runs only on `main`; never deploy from a pull request. See [website/README.md](website/README.md).

## Release intent

Every pull request that changes a published package adds a changeset in `.changeset/`. User-visible changes add a second `user` changeset with a `## User impact` section in plain language. See `docs/releasing.md`.

## Secrets

Never commit keypairs. Program and upgrade-authority keypairs live outside the repository; `*-keypair.json` is ignored.
