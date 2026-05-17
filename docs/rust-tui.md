# Rust TUI

`architect-mcp-tui` is a Ratatui and crossterm terminal client for architect-mcp. It keeps the TypeScript MCP server as the work-gate source of truth, then adds a local UI and headless runner for intake, plan review, adapter readiness, and guarded agent execution.

## Install

The npm package exposes the TUI as a separate binary:

```bash
npx -y --package @tonycdr-prog/architect-mcp architect-mcp-tui
```

The shim never downloads during `postinstall`. It first runs a local built binary when one is present and exposes the current required CLI commands. Stale local binaries that do not support the current smoke surface are skipped. If no usable local binary is found, the shim downloads the matching GitHub release binary into a user cache and verifies the `.sha256` file before execution.

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
answer users=home cooks need controlled agent help
answer coreFlows=grill brief; review plan; promote reviewed changes
answer stack=frontend=Rust Ratatui; backend=TypeScript MCP
answer verification=cargo test --workspace; npm run release:check
grill
contract
review plan
review files
approve run isolated adapter
run adapter
diff summary
diff file <changed-path>
verification status
record verification <required-check>=passed
final review <response>
session review
promotion status
approve promote reviewed diff
promote
arena run codex,shell
arena rank
```

Sessions are persisted under `.architect-mcp/tui/sessions/<id>.json` without secrets.

Use `answer key=value` to fill grill blockers before rerunning `grill`. List-like fields accept semicolon-separated values, for example `coreFlows=grill; review; promote` and `verification=cargo test; npm run release:check`. Stack and repo layout answers accept key pairs, for example `stack=frontend=Rust Ratatui; backend=TypeScript MCP` and `repoLayout=tui=crates/architect-tui/src; docs=docs`.

`approve` is phase-aware. After `review files`, it approves adapter execution only. After adapter evidence, implementation review, and session review are recorded, it approves promotion. Execution approval is cleared after the adapter run, so promotion still needs a separate approval. Use `promotion status` before approving or promoting to see every blocker and next action: missing approval, missing isolated-worktree evidence, missing changed-file evidence, failed verification, missing review gates, or blocking review output.

Verification evidence is strict. The TUI captures the required checks from the live `grill_me` and build-plan gates. Use `verification status` to list the required checks, then use `record verification <check>=passed` with an exact required check name before final review, session review, or promotion approval can proceed. Failed, skipped, not-run, missing, unknown-check, and unknown-status records block the normal path; `override [reason]` remains the explicit maintainer escape hatch.

Adapter run evidence is also strict. Timeout, crash, cancellation, non-zero exit, and truncated output are saved into the session as adapter issues, shown in the inspector, and block normal promotion approval/readiness until the adapter is rerun successfully or a maintainer records an explicit override. A rerun requires file-plan review and execution approval again, replaces the managed isolated worktree for that session/adapter, clears stale run evidence, and then applies the new evidence.

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

Terminal QA smoke:

```bash
architect-mcp-tui smoke --json
```

The smoke command checks help output, adapter readiness, binary SHA-256 and cache metadata, a secret-safe environment summary, and a live gate-only JSONL run. See [Terminal QA](./terminal-qa.md) for platform-specific commands.

Scripted interactive walkthrough:

```bash
architect-mcp-tui walkthrough --json
```

The walkthrough runs the same command-palette engine as the interactive TUI in a throwaway git workspace. It creates a session, answers intake, runs grill/contract/plan/file gates, approves and runs a fixture adapter in an isolated worktree, inspects diff evidence, records verification, runs final/session review, checks `promotion status`, approves promotion, and promotes the reviewed file. It is reproducible release evidence for the operator flow, but it does not replace manual visual terminal QA.

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

On narrow terminals, the inspector moves into a full-width band above the command palette so Agents, Transcript, Inspector, and Command remain visible at the common 80x24 terminal size.

Mouse capture supports layout-aware click, drag, scroll, tab switching, and agent pinning. Approval and promotion are command-palette actions: use `diff summary` and `diff file <path>` to inspect recorded isolated-worktree changes, use `promotion status` to inspect blockers, use `approve [reason]` after review gates pass, then `promote` to copy approved isolated-worktree files back into the workspace. Promotion requires changed-file evidence plus implementation, repo-structure, final-response, and session review gates unless `override [reason]` is used. The TUI never promotes adapter output automatically.

The render scheduler coalesces redraw requests and relies on Ratatui backend diffing instead of clearing the screen after startup. It does not perform true widget-level partial painting.

## Adapters

Built-in adapter templates are Codex, Claude, Gemini, OpenCode, Aider, and a generic shell adapter. Adapters are runtime-probed and show as unavailable when the local CLI is missing. Codex also reports auth state from `codex login status`; it is ready only when the command succeeds and reports `Logged in`. Other authenticated CLIs remain `auth unknown` until reliable probes are added.

Agents run in a PTY by default when execution is explicitly enabled. Headless `--execute` creates an isolated git worktree, streams PTY output, records changed-file evidence, and runs `review_implementation_against_contract`, `review_repo_structure`, `review_agent_final_response`, and `review_agent_session` before any promotion. PTY output is capped with an explicit truncation marker, and truncated output becomes a promotion blocker in the normal path.

The `arena run <adapter[,adapter]>` command runs named adapters against the current contract into isolated worktrees and records each candidate's diff evidence. The `arena rank` command ranks recorded candidates with a deterministic score based on implementation review status, verification status, diff size, contract drift, and crash state. Candidate promotion remains manual; the arena never auto-merges a winner.

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

Cross-platform install smoke, live-QA smoke, and scripted interactive walkthrough checks run in GitHub Actions on Linux, macOS, and Windows. Manual terminal checks are described in [Terminal QA](./terminal-qa.md), and release-candidate evidence is tracked in [TUI Live QA](./tui-live-qa.md).
