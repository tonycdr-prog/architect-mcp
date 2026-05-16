# Hosted Mode

Hosted HTTP mode is stateless and does not need a database. Each `/mcp` request creates a fresh MCP server with no session storage.

```bash
npm install
npm run build
PORT=3000 HOST=0.0.0.0 npm run start:http
```

## Endpoints

- `POST /mcp`: stateless Streamable HTTP MCP endpoint.
- `GET /health`: simple health check for deploy platforms.

## Safety Boundary

Hosted mode intentionally excludes local-only tools, even when the advanced surface is enabled:

- `promote_stack_pack_to_files`
- `review_local_workspace`
- `scan_mcp_config_files`
- `apply_mcp_install_plan`

Use `review_repo_structure` instead by sending file summaries from the client or agent. Use `review_mcp_config_security` with parsed config objects instead of asking the hosted server to scan local config files.

Hosted advanced mode may return MCP server recommendations and dry-run install plans. It must not write MCP client configuration; `apply_mcp_install_plan` is excluded from hosted HTTP registration.

Unknown tools classify as `unknown` in policy audits and must not be treated as hosted-safe.

## File Summaries

For existing repos, include both source file summaries and directory paths. Directory paths let the reviewer validate mapped folders that may not contain reviewed source files.

```json
{
  "path": "src/features/billing/BillingClient.tsx",
  "lines": 180,
  "imports": ["@/server/billing"],
  "hasUseClient": true,
  "envAccesses": [],
  "hasDirectDbAccess": false
}
```

Local workspace scans derive those signals automatically. Hosted clients must send them explicitly.

## Review Modes

- `strict`: return every finding.
- `summary`: return grouped findings plus priority findings.
- `ci`: fail on new error findings, with optional baseline suppression.
- `migration`: lower-noise adoption mode for mature repos with known debt.
- `audit`: source findings without requiring all repo hygiene files to exist yet.

## Environment Variables

- `PORT`: HTTP port. Defaults to `3000`.
- `HOST`: bind host. Defaults to `0.0.0.0`.
- `ALLOWED_HOSTS`: comma-separated host allow-list for DNS rebinding protection.
- `JSON_BODY_LIMIT`: maximum JSON request size for large repo reviews. Defaults to `10mb`.

Railway defaults are included in `railway.json`:

```bash
npm ci && npm run build
npm run start:http
```

## API Shape

The future hosted API shape is documented in [Hosted API Shape](./hosted-api-shape.md). Those routes are contract targets, not a stateful hosted product implementation.
