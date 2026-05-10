# Release Readiness

The release gate is:

```bash
npm run release:check
```

That command chains the advanced staged readiness gate through `npm run check:v10`. The base readiness path runs typecheck, tests, build, docs build, audit, package dry-run checks, package manifest hygiene, and `mcp_readiness_report`.

## Local Checks

Run these before opening a release-sensitive PR:

```bash
npm run docs:build
npm run typecheck
npm test
npm run build
npm run release:check
```

## Package Boundary

The package includes runtime output and docs:

- `dist/`
- `packs/`
- `policy-bundles/`
- `foundation-packs/`
- selected `stack-sources/` metadata
- `examples/`
- `docs/`
- `llms.txt`
- `README.md`
- `LICENSE`
- `SECURITY.md`
- `.env.example`

Repo-only TypeScript readiness scripts and docs development scripts are stripped from the packed `package.json` during `prepack`.

## npm Publishing

Publishing is handled by `.github/workflows/npm-publish.yml` when a GitHub release is published from a `v*` tag. The workflow:

- Uses pinned GitHub Actions.
- Installs with Node 22 and `npm ci`.
- Runs `npm run release:check`.
- Packs the package and installs the tarball in a temporary project.
- Publishes with `npm publish --access public --provenance`.

The repository must define `NPM_TOKEN` with permission to publish `@tonycdr-prog/architect-mcp`. Do not bypass `npm run release:check`; it remains the clean-checkout release gate.

## GitHub Pages

The docs site builds with VitePress from the existing `docs/` directory and uses `base: "/architect-mcp/"`. The Pages workflow runs on pushes to `main` and manual dispatch, installs with Node 22, builds with `npm run docs:build`, uploads `docs/.vitepress/dist`, and deploys via GitHub Pages Actions.

After merge, repository Pages settings should use "GitHub Actions" as the Pages source.

## Compatibility Gate Names

Historical `check:v3` through `check:v10` scripts remain because they protect compatibility and release confidence. Public docs describe these checks as advanced maturity, governance, operating-model, and productization boundary criteria rather than public product versions.
