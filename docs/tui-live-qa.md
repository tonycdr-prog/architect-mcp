# TUI Live QA

Use this page for release-candidate validation that cannot be proven by unit tests alone. Do not publish a TUI release until the clean release gate passes and the platform smoke matrix is green or explicitly waived.

## Automated Matrix

The `.github/workflows/tui-install-smoke.yml` workflow runs on pull requests and manual dispatch across:

- Ubuntu latest.
- macOS 14.
- Windows latest.

Each job installs Node dependencies, installs the pinned Rust toolchain, builds the release binary, runs the npm shim help command, and runs shim tests for local binary resolution, cached binary reuse, missing binary failure, and checksum mismatch failure.

## Manual Matrix

Before marking a TUI release ready, run at least one live workflow on each target family:

| Platform | Required checks |
| --- | --- |
| macOS | `npm run release:check`, `npm run tui:build`, `architect-mcp-tui config adapters --json`, gate-only `run --jsonl`, and one isolated-worktree `--execute` with a safe shell adapter |
| Linux | `npm ci`, `npm run tui:build`, `node bin/architect-mcp-tui.cjs --help`, gate-only `run --jsonl`, and adapter health output |
| Windows | `npm ci`, `npm run tui:build`, `node bin/architect-mcp-tui.cjs --help`, shim tests, and `architect-mcp-tui config adapters --json` |

## Live Workflow Checklist

Use a fresh private repository or local throwaway git repo. Do not run destructive commands in public repositories.

1. Confirm `architect-mcp-tui config adapters --json` reports Codex auth accurately.
2. Run a vague prompt in gate-only mode and confirm it stops at `approval_required` after `grill_me`.
3. Run a ready prompt through `grill_me`, `create_pre_edit_contract`, `review_build_plan`, and `review_proposed_file_plan` without `--execute`; confirm no adapter process starts.
4. Run a safe shell adapter with `--execute` in an isolated worktree; confirm JSONL remains parseable and includes `diff_evidence`.
5. Confirm implementation review, repo-structure review, final-response review, and session review events appear before `review_required`.
6. In the interactive TUI, create a session, run `approve <reason>`, run `promote`, and confirm only the changed files from `.architect-mcp/worktrees/<session>/<adapter>` are copied.
7. Run `arena rank` and confirm candidates are ranked without auto-merging.
8. Cancel one interactive session and confirm persisted session JSON records cancellation without secrets.

## Evidence To Record

For each platform, record:

- OS and CPU architecture.
- Node, npm, Rust, and `architect-mcp-tui` versions.
- Adapter health JSON summary.
- Commands run and pass/fail status.
- Any terminal rendering corruption, mouse-routing issue, or JSONL parse failure.
- Whether promotion was tested and which files were promoted.

## Release Decision

Go only when:

- `npm run release:check` passes from a clean checkout.
- The TUI install-smoke workflow is green.
- Linux, macOS, and Windows manual smoke evidence is recorded or a maintainer explicitly waives a platform with reason.
- No Copilot review blocker remains on the release PR.
