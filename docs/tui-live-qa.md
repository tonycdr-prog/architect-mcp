# TUI Live QA

Use this page for release-candidate validation that cannot be proven by unit tests alone. Do not publish a TUI release until the clean release gate passes and the platform smoke matrix is green or explicitly waived.

For public tester commands and issue templates, use [Terminal QA](./terminal-qa.md).

## Automated Matrix

The `.github/workflows/tui-install-smoke.yml` and `.github/workflows/tui-live-qa.yml` workflows run on pull requests and manual dispatch across:

- Ubuntu latest.
- macOS 14.
- Windows latest.

The install-smoke workflow installs Node dependencies, installs the pinned Rust toolchain, builds the release binary, runs the npm shim help command, and runs shim tests for local binary resolution, cached binary reuse, missing binary failure, and checksum mismatch failure. The live-QA smoke workflow runs `npm run tui:live-qa`, which builds a local debug TUI binary before exercising the shim help path and workflow tests covering headless JSONL, approval failure handling, and focused diff commands on every OS. PTY adapter execution and multi-candidate arena evidence run in the Unix matrix until the Windows portable PTY path has stable hosted-runner evidence. The post-release smoke workflow runs on Ubuntu and Windows only and validates `architect-mcp-tui --help`, `config adapters --json` (with JSON parse assertion), and a gate-only `run --jsonl` (with per-line JSON parse assertion) to confirm the binary and MCP gate path work on both platforms without a real terminal.

## Post-Release Evidence

Latest recorded release: `v0.2.1`, published on 2026-05-16.

| Evidence | Platform coverage | Status | Link or note |
| --- | --- | --- | --- |
| npm publish workflow for `v0.2.1` | Ubuntu publish runner | Passed | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25968887248 |
| TUI install-smoke workflow | Ubuntu, macOS 14, Windows latest | Passed on latest recorded PR run | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25970418642 |
| TUI live-QA smoke workflow | Ubuntu, macOS 14, Windows latest | Passed on latest recorded PR run | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25969825040 |
| TUI post-release smoke workflow | Ubuntu, Windows latest | See workflow runs | `.github/workflows/tui-post-release-smoke.yml` — validates `--help`, `config adapters --json`, and gate-only `run --jsonl` on each platform |
| Manual macOS source-checkout smoke | macOS local terminal | Record in the release PR or release notes when run | Maintainer evidence required before a TUI production go decision |
| Manual Linux terminal smoke | Linux local or VM terminal | Pending unless explicitly waived | Hosted workflow evidence is not a substitute for a real terminal smoke |
| Manual Windows terminal smoke | Windows Terminal or PowerShell | Pending unless explicitly waived | Hosted workflow evidence is not a substitute for a real terminal smoke |

Do not convert pending manual rows to passed status without platform, command, version, and failure-note evidence.

## Manual Matrix

Before marking a TUI release ready, run at least one live workflow on each target family:

| Platform | Required checks |
| --- | --- |
| macOS | `npm run release:check`, `npm run tui:build`, `architect-mcp-tui config adapters --json`, gate-only `run --jsonl`, and one isolated-worktree `--execute` with a safe shell adapter |
| Linux | `npm ci`, `npm run tui:build`, `node bin/architect-mcp-tui.cjs smoke --json`, and one optional interactive launch |
| Windows | `npm ci`, `npm run tui:build`, `node bin/architect-mcp-tui.cjs smoke --json`, shim tests, and one optional interactive launch |

## Live Workflow Checklist

Use a fresh private repository or local throwaway git repo. Do not run destructive commands in public repositories.

1. Confirm `architect-mcp-tui config adapters --json` reports Codex auth accurately.
1. Run `architect-mcp-tui smoke --json` and save the report.
2. Run a vague prompt in gate-only mode and confirm it stops at `approval_required` after `grill_me`.
3. Run a ready prompt through `grill_me`, `create_pre_edit_contract`, `review_build_plan`, and `review_proposed_file_plan` without `--execute`; confirm no adapter process starts.
4. Run a safe shell adapter with `--execute` in an isolated worktree; confirm JSONL remains parseable and includes `diff_evidence`.
5. Confirm implementation review, repo-structure review, final-response review, and session review events appear before `review_required`.
6. In the interactive TUI, create a session, run `diff summary`, run `diff file <path>`, run `approve <reason>`, run `promote`, and confirm only the changed files from `.architect-mcp/worktrees/<session>/<adapter>` are copied.
7. Run `arena run <adapter[,adapter]>`, then `arena rank`, and confirm candidates are ranked without auto-merging.
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
