# GitHub Copilot Instructions

This repository implements: Local-first MCP standards and verification harness for coding agents. It grills project briefs, generates architecture contracts, creates repo artifacts, reviews implementation drift, and enforces evidence before completion.

## Commands
- `npm run typecheck`
- `npm test`
- `npm run build`
- `npm run check:v10`

## Architecture Boundaries
- Domain decisions live in `src/domain/*`.
- MCP registration, schemas, and response formatting live in `src/tools/*`.
- Server construction and transport boundaries live in `src/server/*`, `src/http.ts`, and `src/index.ts`.
- Filesystem and host adapters live in `src/infrastructure/*`.
- Tests live in `tests/*`.

## Review Rules
- Do not create giant aggregation files or monolithic route/tool modules.
- Do not add hosted, database, GitHub, billing, account, or dashboard runtime behavior unless the current task explicitly targets that layer.
- Keep local MCP safety behavior available without paid, hosted, or remote services.
- Require evidence before claiming a root cause or marking work complete.
- Update generated artifact tests when changing repo artifact generation.

## Current Contract
- Contract version: 0.2.0
- Stack packs: mcp-server
- Foundation packs: agent-harness, ci-gates, repo-hygiene, repo-structure, testing
