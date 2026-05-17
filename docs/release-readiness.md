# Release Readiness

The release gate is:

```bash
npm run release:check
```

That command runs `npm run rust:check`, then chains the advanced staged readiness gate through `npm run check:v10`. The base readiness path runs typecheck, tests, build, docs build, audit, package dry-run checks, package manifest hygiene, and `mcp_readiness_report`.

## Local Checks

Run these before opening a release-sensitive PR:

```bash
npm run docs:build
npm run rust:check
npm run typecheck
npm test
npm run build
npm run release:check
```

## Package Boundary

The package includes runtime output and docs:

- `dist/`
- `bin/architect-mcp-tui.cjs`
- Rust TUI workspace sources and lockfile
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
- Installs with Node 24, upgrades npm to `^11.5.1` for provenance-capable publishing, and runs `npm ci`.
- Installs the pinned Rust 1.94.0 toolchain.
- Runs `npm run release:check`.
- Packs the package and installs the tarball in a temporary project.
- Publishes with `npm publish --access public --provenance`.

The repository must define `NPM_TOKEN` with permission to publish `@tonycdr-prog/architect-mcp`. For 2FA-protected npm accounts, use a granular token that can publish the package and bypass 2FA for automation. The workflow also uses GitHub Actions OIDC and npm 11.5.1+ for provenance-capable publishing. Do not bypass `npm run release:check`; it remains the clean-checkout release gate.

### Trusted Publishing Migration

npm trusted publishing can remove the long-lived publish token once the package is configured on npmjs.com. The repository workflow already has the release-side prerequisites: GitHub-hosted runner, `id-token: write`, Node 24, npm `^11.5.1`, provenance-capable publish, and the clean `npm run release:check` gate.

The remaining setup happens in npm package settings, not in this repository:

1. Open `@tonycdr-prog/architect-mcp` on npmjs.com.
2. In package settings, add a trusted publisher for GitHub Actions.
3. Use owner `tonycdr-prog`, repository `architect-mcp`, and workflow filename `npm-publish.yml`.
4. Run the next release without changing the release gate.
5. After a successful trusted-publishing release, restrict publishing access to require 2FA and disallow traditional tokens if that policy fits the maintainer account.
6. Revoke the old automation token.

Until that npm-side configuration is complete, keep `NPM_TOKEN` in GitHub secrets so releases remain publishable. Do not commit tokens, paste token values into issues, or place token material in workflow logs.

### Token Rotation

If token-backed publishing is still enabled, rotate the token after any suspected exposure, maintainer handoff, or release-process change:

1. Create a new granular npm token scoped to `@tonycdr-prog/architect-mcp` with publish rights and the required 2FA automation setting.
2. Update the GitHub Actions `NPM_TOKEN` secret.
3. Run `npm run release:check` locally from a clean checkout.
4. Publish the next tag or rerun the publish workflow for the intended release.
5. Verify `npm view @tonycdr-prog/architect-mcp version` and a fresh `npx -y --package @tonycdr-prog/architect-mcp architect-mcp --help` install path.
6. Revoke the previous automation token.

## TUI Release Binaries

`.github/workflows/tui-release.yml` builds `architect-mcp-tui` for Linux x64, Linux ARM64, macOS x64, macOS ARM64, and Windows x64 release assets. Each archive is uploaded with a `.sha256` checksum. The npm shim downloads only matching release assets and verifies the checksum before execution.

The shim follows HTTPS redirects for GitHub release asset and checksum downloads. `.github/workflows/tui-install-smoke.yml` runs the shim and release-binary build path across Ubuntu x64, Ubuntu ARM64, macOS, and Windows on pull requests and manual dispatch. `.github/workflows/tui-live-qa.yml` adds cross-platform TUI workflow smoke coverage. Published-package ARM64 smoke starts after a release containing the Linux ARM64 asset exists; until then, keep hosted published-package smoke on the released x64/Windows asset set. Manual OS evidence is tracked in [TUI Live QA](./tui-live-qa.md).

`.github/workflows/governance-audit.yml` runs the read-only governance audit on manual dispatch and a weekly schedule. It writes a public-safe GitHub step summary rather than uploading raw local audit JSON. Governance reports and redaction rules live in [Governance Audit](./governance-audit.md).

## GitHub Pages

The docs site builds with VitePress from the existing `docs/` directory and uses `base: "/architect-mcp/"`. The Pages workflow runs on pushes to `main` and manual dispatch, installs with Node 22, builds with `npm run docs:build`, uploads `docs/.vitepress/dist`, and deploys via GitHub Pages Actions.

After merge, repository Pages settings should use "GitHub Actions" as the Pages source.

## Compatibility Gate Names

Historical `check:v3` through `check:v10` scripts remain because they protect compatibility and release confidence. Public docs describe these checks as advanced maturity, governance, operating-model, and productization boundary criteria rather than public product versions.
