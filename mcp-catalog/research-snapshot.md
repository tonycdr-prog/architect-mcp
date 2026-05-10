# MCP Catalog Research Snapshot

Date: 2026-05-11

This snapshot records the source basis for the initial guarded MCP recommendation catalog. It is not an exhaustive ecosystem index.

## Source Basis

- Official MCP Registry: preview centralized metadata repository for public MCP servers, namespace verification, registry API, and standardized install metadata.
- modelcontextprotocol/servers: reference implementations and examples, explicitly not enough by itself as production-readiness proof.
- Supabase MCP docs: vendor-hosted MCP endpoint, OAuth flow, read-only option, development/test data guidance, and security risk notes.
- Stripe MCP docs: hosted MCP endpoint, OAuth sessions, restricted-key guidance, and local MCP package example.
- Notion MCP docs: vendor-hosted endpoint, OAuth setup, and Codex TOML configuration shape.
- Railway MCP docs: CLI local stdio server and remote MCP endpoint, with supported client install targets including Codex.
- CLI Printing Press and Printing Press Library: MIT ecosystem research for generated CLI plus MCP companion tooling, not a default install target.

## Package Versions Checked

- `@supabase/mcp-server-supabase`: `0.8.1`
- `@stripe/mcp`: `0.3.3`
- `@notionhq/notion-mcp-server`: `2.2.1`
- `@railway/mcp-server`: `0.1.8`
- `@playwright/mcp`: `0.0.75`
- `@modelcontextprotocol/server-memory`: `2026.1.26`
- `@modelcontextprotocol/server-github`: `2025.4.8`

Hosted vendor endpoints are preferred in the catalog where current vendor docs present OAuth-based remote MCP setup. Package versions are retained for local stdio plans where the install plan uses `npx`.

## Guardrails

- Unknown servers fail closed.
- Database recommendations require provider confirmation.
- Payments recommendations require provider confirmation plus a server-side boundary.
- Local config writes remain local-only and dry-run by default.
- Secrets are referenced through placeholders only.
- Package-backed install commands must pin exact versions.
