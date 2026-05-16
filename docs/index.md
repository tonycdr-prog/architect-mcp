---
layout: home

hero:
  name: architect-mcp
  text: Agent work gate for coding agents
  tagline: Clarify before edits, constrain the plan, review implementation drift, and require verification evidence before completion.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started
    - theme: alt
      text: Core Flow
      link: /core-work-gate
    - theme: alt
      text: Tool Reference
      link: /tool-reference
    - theme: alt
      text: Rust TUI
      link: /rust-tui

features:
  - title: Default work gate
    details: The default MCP surface exposes eight tools focused on intake, pre-edit contracts, plan review, drift review, and final response honesty.
  - title: Local-first by default
    details: Local workspace scanning and file-writing helpers stay local-only. Hosted mode is stateless and accepts explicit file summaries.
  - title: Advanced maturity criteria
    details: Stack packs, governed MCP install plans, governance evals, operating-model evals, repo-quality gates, and productization boundary checks remain available behind the advanced surface.
  - title: Rust TUI platform
    details: Ratatui brings the live work gate into a mouse-aware terminal client with adapter readiness, guarded headless JSONL, approval/promotion commands, arena ranking, and provisional ACP stdio mode.
  - title: Release-gated
    details: The clean-checkout release gate is npm run release:check, which includes Rust checks, typecheck, tests, build, docs build, audit, package dry-run checks, and readiness reports.
---

## Product Story

architect-mcp is a TypeScript MCP server that keeps coding agents inside a deliberate work loop:

1. Pressure-test the request with `grill_me`.
2. Capture a pre-edit contract before risky changes.
3. Review the build plan and proposed file plan before writing files.
4. Review implementation drift and repo structure after the change.
5. Require honest final-response evidence.

The first-run product surface is intentionally small. The advanced surface is for teams that want the full standards, pack-authoring, governance, repo-quality, and eval layers.

`architect-mcp-tui` adds a local terminal client for the same loop without rewriting the TypeScript MCP server.

## Distribution

The command remains `architect-mcp`. The published package identity is scoped as `@tonycdr-prog/architect-mcp`, and the current install path uses GitHub release tarballs until npm publishing is configured for that scope.

```bash
npm install -g https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.1/tonycdr-prog-architect-mcp-0.1.1.tgz
architect-mcp
```

## Key References

- [Getting Started](/getting-started)
- [MCP Client Setup](/mcp-client-setup)
- [Core Work Gate](/core-work-gate)
- [Hosted Mode](/hosted-mode)
- [MCP Integrations](/mcp-integrations)
- [Rust TUI](/rust-tui)
- [TUI Live QA](/tui-live-qa)
- [Release Readiness](/release-readiness)
- [Read-Only Smoke Matrix](/read-only-smoke-matrix)
- [Compatibility And Advanced Maturity Criteria](/compatibility)
