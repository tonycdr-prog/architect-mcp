# TUI Live QA

Use this page for release-candidate validation that cannot be proven by unit tests alone. Do not publish a TUI release until the clean release gate passes and the platform smoke matrix is green or explicitly waived.

For public tester commands and issue templates, use [Terminal QA](./terminal-qa.md). Linux and Windows Terminal QA reports should include public-safe launch judge evidence generated with `architect-mcp-tui terminal-evidence --json` and consumable with `architect-mcp-tui launch-judge --terminal-evidence`; they should not include raw smoke JSON, raw stdout/stderr logs, absolute local paths, cache paths, private repo names, or tokens. For maintained-repo drift and governance reports, use [Governance Audit](./governance-audit.md).

## Automated Matrix

The `.github/workflows/tui-install-smoke.yml` and `.github/workflows/tui-live-qa.yml` workflows run on pull requests and manual dispatch across:

- Ubuntu latest.
- macOS 14.
- Windows latest.

The install-smoke workflow installs Node dependencies, installs the pinned Rust toolchain, builds the release binary, runs the npm shim help command, and runs shim tests for local binary resolution, cached binary reuse, missing binary failure, and checksum mismatch failure. The live-QA smoke workflow runs `npm run tui:live-qa`, which builds the TypeScript MCP server, builds a local debug TUI binary, exercises the shim help path, runs `architect-mcp-tui walkthrough --json`, runs fixture-backed promotion-smoke unit coverage, and runs workflow tests covering headless JSONL, approval failure handling, and focused diff commands on every OS. Real Codex promotion smoke is manual because it requires local auth and a live model. PTY adapter execution and multi-candidate arena evidence run in the Unix matrix until the Windows portable PTY path has stable hosted-runner evidence.

## Post-Release Evidence

Latest recorded release: `v0.2.1`, published on 2026-05-16.

| Evidence | Platform coverage | Status | Link or note |
| --- | --- | --- | --- |
| npm publish workflow for `v0.2.1` | Ubuntu publish runner | Passed | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25968887248 |
| TUI install-smoke workflow | Ubuntu, macOS 14, Windows latest | Passed on latest recorded PR run | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25970418642 |
| TUI live-QA smoke workflow | Ubuntu, macOS 14, Windows latest | Passed on latest recorded PR run | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25969825040 |
| Manual macOS source-checkout smoke | macOS local terminal | Record in the release PR or release notes when run | Maintainer evidence required before a TUI production go decision |
| Manual Linux terminal smoke | Linux local or VM terminal | Pending unless explicitly waived | Hosted workflow evidence is not a substitute for a real terminal smoke |
| Manual Windows terminal smoke | Windows Terminal or PowerShell | Pending unless explicitly waived | Hosted workflow evidence is not a substitute for a real terminal smoke |

Do not convert pending manual rows to passed status without platform, command, version, and failure-note evidence.

When Linux or Windows manual evidence is posted publicly, prefer the generated evidence command:

```bash
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
```

When Linux and Windows reports are generated separately, pass both files to the launch judge:

```bash
architect-mcp-tui launch-judge --json --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
```

If a maintainer must normalize already-posted notes manually, keep the same launch judge terminal evidence schema:

```json
{
  "schemaVersion": 1,
  "reports": [
    {
      "platform": "linux",
      "status": "passed",
      "source": "issue #136 public-safe terminal QA report",
      "commandSummary": "architect-mcp-tui --help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "summary only, no raw logs"
    }
  ]
}
```

## Manual Matrix

Before marking a TUI release ready, run at least one live workflow on each target family:

| Platform | Required checks |
| --- | --- |
| macOS | `npm run release:check`, `npm run tui:build`, `architect-mcp-tui config adapters --json`, gate-only `run --jsonl`, and one isolated-worktree `--execute` with a safe shell adapter |
| Linux | `npm ci`, `npm run tui:build`, `node bin/architect-mcp-tui.cjs smoke --json`, and one optional interactive launch |
| Windows | `npm ci`, `npm run tui:build`, `node bin/architect-mcp-tui.cjs smoke --json`, shim tests, and one optional interactive launch |

The npm shim checks that a local source-built binary exposes the current required command surface before using it. If a stale `target/release` binary is present but does not support `smoke`, the shim should skip it and use a newer usable local binary or the verified release-cache path.

## Live Workflow Checklist

Use a fresh private repository or local throwaway git repo. Do not run destructive commands in public repositories.

1. Confirm `architect-mcp-tui config adapters --json` reports Codex auth accurately.
1. Run `architect-mcp-tui smoke --json` and save the report.
1. Run `architect-mcp-tui walkthrough --json` and confirm it ends with `status=passed`, `finalApproval=promoted`, and at least one promoted file.
1. When Codex is authenticated, run `architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace` and confirm it ends with `status=passed`, `finalApproval=promoted`, and only `docs/codex-adapter-smoke.md` promoted.
2. Run a vague prompt in gate-only mode and confirm it stops at `approval_required` after `grill_me`.
3. Run a ready prompt through `grill_me`, `create_pre_edit_contract`, `review_build_plan`, and `review_proposed_file_plan` without `--execute`; confirm no adapter process starts.
4. Run a safe shell adapter with `--execute` in an isolated worktree; confirm JSONL remains parseable and includes `diff_evidence`.
5. Confirm implementation review, repo-structure review, final-response review, and session review events appear before `review_required`.
6. In the interactive TUI, create a session, run `diff summary`, run `diff file <path>`, run `promotion status`, run `approve <reason>`, run `promote`, and confirm only the changed files from `.architect-mcp/worktrees/<session>/<adapter>` are copied.
7. After file-plan review, approve arena execution, run `arena run <adapter[,adapter]>`, then `arena rank`, and confirm candidates are ranked without auto-merging.
8. Run `arena select <adapter>` for one candidate and confirm promotion still requires verification, final/session review, and a separate promotion approval before files are copied.
9. Run `integrations recommend` for a generic database request and confirm it asks for a provider before Supabase can be planned.
10. After explicitly setting a provider, run `integrations plan <server>`, `integrations review`, `integrations apply`, `integrations approve <reason>`, and `integrations write`, and confirm the dry run does not write while the final write requires approval.
11. Cancel one interactive session and confirm persisted session JSON records cancellation without secrets.

## Evidence To Record

For each platform, record:

- OS and CPU architecture.
- Node, npm, Rust, and `architect-mcp-tui` versions.
- Adapter health JSON summary.
- Commands run and pass/fail status.
- Any terminal rendering corruption, mouse-routing issue, or JSONL parse failure.
- Scripted walkthrough status and promoted files.
- Whether fixture promotion and real-adapter promotion were tested, and which files were promoted.

## Release Decision

Go only when:

- `npm run release:check` passes from a clean checkout.
- The TUI install-smoke workflow is green.
- Linux, macOS, and Windows manual smoke evidence is recorded or a maintainer explicitly waives a platform with reason.
- No Copilot review blocker remains on the release PR.
