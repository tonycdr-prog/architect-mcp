# Rust TUI

`architect-mcp-tui` is a Ratatui and crossterm terminal client for architect-mcp. It keeps the TypeScript MCP server as the work-gate source of truth, then adds a local UI and headless runner for intake, plan review, adapter readiness, and guarded agent execution.

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

The command palette supports the guarded app-building path:

```text
new app <idea>
answer users=home cooks
grill
contract
review plan
review files
run adapter
record verification npm test=passed
final review <response>
session review
```

Sessions are persisted under `.architect-mcp/tui/sessions/<id>.json` without secrets.

Headless JSONL run:

```bash
architect-mcp-tui run --prompt "Build an offline recipe planner" --adapter codex --jsonl
```

Headless mode is gate-only by default. It calls live architect-mcp tools and stops before adapter execution unless `--execute` is supplied:

```bash
architect-mcp-tui run --prompt "Build an offline recipe planner" --adapter codex --jsonl --execute
```

ACP stdio server:

```bash
architect-mcp-tui acp --stdio
```

ACP mode is currently a provisional JSON-RPC compatibility surface for early client testing, not a full ACP conformance claim.

Config commands:

```bash
architect-mcp-tui config init
architect-mcp-tui config doctor
architect-mcp-tui config adapters
architect-mcp-tui config adapters --json
```

## Work Gate

Every coding and app-building loop starts with the architect-mcp work gate:

1. `grill_me`
2. `create_pre_edit_contract`
3. `review_build_plan`
4. `review_proposed_file_plan`
5. adapter execution only after explicit approval or `--execute`
6. `review_implementation_against_contract`
7. verification evidence
8. `review_agent_final_response`
9. `review_agent_session`

The headless runner starts architect-mcp with `ARCHITECT_MCP_TOOL_SURFACE=advanced`, calls `grill_me` over stdio first, and stops with `approval_required` when the brief is incomplete. If the brief is ready, it calls `create_pre_edit_contract`, `review_build_plan`, and `review_proposed_file_plan`, then pauses before any adapter process unless `--execute` is present.

## Interface

The main layout has four surfaces:

- Left: session and agent tree.
- Center: transcript and plan panel.
- Right: inspector for gates, adapters, and approval state.
- Bottom: command palette and prompt input.

Mouse capture supports layout-aware click, drag, scroll, tab switching, and agent pinning. Approval buttons and diff promotion are planned workflow actions, not release-ready behavior yet.

The render scheduler coalesces redraw requests and relies on Ratatui backend diffing instead of clearing the screen after startup. It does not perform true widget-level partial painting.

## Adapters

Built-in adapter templates are Codex, Claude, Gemini, OpenCode, Aider, and a generic shell adapter. Adapters are runtime-probed and show as unavailable when the local CLI is missing. Codex also reports auth state from `codex login status`; it is ready only when the command succeeds and reports `Logged in`. Other authenticated CLIs remain `auth unknown` until reliable probes are added.

Agents run in a PTY by default when execution is explicitly enabled. Headless `--execute` creates an isolated git worktree, streams PTY output, records changed-file evidence, and runs `review_implementation_against_contract`, `review_repo_structure`, `review_agent_final_response`, and `review_agent_session` before any promotion. PTY output is capped with an explicit truncation marker. Parallel arena ranking and promotion flows are still production-readiness follow-up work.

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
