# Rust TUI

`architect-mcp-tui` is a Ratatui and crossterm terminal client for architect-mcp. It keeps the TypeScript MCP server as the work-gate source of truth, then adds a local UI for intake, plan review, file-plan approval, adapter execution, verification, and final/session review.

## Install

The npm package exposes the TUI as a separate binary:

```bash
npx -y --package @tonycdr-prog/architect-mcp architect-mcp-tui
```

The shim never downloads during `postinstall`. It first runs a local built binary when one is present. If not, it downloads the matching GitHub release binary into a user cache and verifies the `.sha256` file before execution.

For a source checkout:

```bash
npm install
npm run tui:build
node bin/architect-mcp-tui.cjs
```

## CLI

Interactive TUI:

```bash
architect-mcp-tui
```

Headless JSONL run:

```bash
architect-mcp-tui run --prompt "Build an offline recipe planner" --adapter codex --jsonl
```

ACP stdio server:

```bash
architect-mcp-tui acp --stdio
```

Config commands:

```bash
architect-mcp-tui config init
architect-mcp-tui config doctor
architect-mcp-tui config adapters
```

## Work Gate

Every coding and app-building loop is routed through the architect-mcp work gate:

1. `grill_me`
2. `create_pre_edit_contract`
3. `review_build_plan`
4. `review_proposed_file_plan`
5. adapter execution
6. `review_implementation_against_contract`
7. verification evidence
8. `review_agent_final_response`
9. `review_agent_session`

The TUI starts architect-mcp with `ARCHITECT_MCP_TOOL_SURFACE=advanced` so the guarded Integrations panel can access catalog and install-plan review tools while local-only tools stay behind local execution boundaries.

## Interface

The main layout has four surfaces:

- Left: session and agent tree.
- Center: transcript, plan, and diff panel.
- Right: inspector for gates, adapters, and approval state.
- Bottom: command palette and prompt input.

Mouse capture supports click, drag, scroll, tab switching, agent pinning, and approval actions. The renderer coalesces dirty widgets and relies on Ratatui backend diffing instead of clearing the screen after startup.

## Adapters

Built-in adapter templates are Codex, Claude, Gemini, OpenCode, Aider, and a generic shell adapter. Adapters are runtime-probed and show as unavailable when the local CLI is missing.

Agents run in a PTY by default. Parallel and arena flows use isolated git worktrees by default. Shared-workspace mode requires explicit confirmation in the UI.

## Config

Repo config lives at:

```text
.architect-mcp/tui.toml
```

User config lives at:

```text
~/.config/architect-mcp/tui.toml
```

Repo config may override adapters only inside the workspace. Secrets must come from environment variables, not config values.

## Release Gate

The release gate now includes the Rust checks before the existing maturity gate:

```bash
npm run release:check
```

Rust-only checks:

```bash
npm run rust:check
```

That runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
