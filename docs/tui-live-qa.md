# TUI Live QA

Use this page for release-candidate validation that cannot be proven by unit tests alone. Do not publish a TUI release until the clean release gate passes and the platform smoke matrix is green or explicitly waived.

For public tester commands and issue templates, use [Terminal QA](./terminal-qa.md). Linux and Windows Terminal QA reports should include public-safe launch judge evidence generated with `architect-mcp-tui terminal-evidence --markdown`, including the generated `collectedAt` date and `environment` provenance, collected with `architect-mcp-tui collect-terminal-evidence --json`, and consumable with `architect-mcp-tui launch-judge --terminal-evidence`; they should not include raw smoke JSON, raw stdout/stderr logs, absolute local paths, cache paths, private repo names, or tokens. For maintained-repo drift and governance reports, use [Governance Audit](./governance-audit.md).

## Automated Matrix

The `.github/workflows/tui-install-smoke.yml` and `.github/workflows/tui-live-qa.yml` workflows run on pull requests and manual dispatch across:

- Ubuntu latest.
- Ubuntu 24.04 ARM.
- macOS 14.
- Windows latest.

The install-smoke workflow installs Node dependencies, installs the pinned Rust toolchain, builds the release binary, runs the npm shim help command, and runs shim tests for local binary resolution, cached binary reuse, missing binary failure, and checksum mismatch failure. The live-QA smoke workflow runs `npm run tui:live-qa`, which builds the TypeScript MCP server, builds a local debug TUI binary, exercises the shim help path, runs `architect-mcp-tui walkthrough --json`, emits a public launch-judge summary instead of the full local launch report, runs fixture-backed promotion-smoke unit coverage, and runs workflow tests covering headless JSONL, approval failure handling, and focused diff commands on every OS. On Ubuntu and Windows, the workflow also writes a public-safe `architect-mcp-tui terminal-evidence --json` summary to the GitHub step summary as hosted CI baseline evidence. That hosted evidence is not a substitute for the real post-release terminal QA tracked in #136. Real Codex promotion smoke is manual because it requires local auth and a live model. PTY adapter execution and multi-candidate arena evidence run in the Unix matrix until the Windows portable PTY path has stable hosted-runner evidence.

The `.github/workflows/tui-published-package-smoke.yml` workflow is the published-package counterpart. It does not check out or build this repository; it installs `@tonycdr-prog/architect-mcp@latest` into a clean temporary npm workspace on hosted Ubuntu and Windows, points the TUI bridge at the installed package's `dist/index.js`, runs the documented non-interactive TUI commands, validates adapter-health JSON, and verifies the gate-only JSONL run stops at `approval_required` without adapter execution. This is useful release telemetry for the package users actually install, but it does not replace #136 manual terminal QA because hosted runners do not validate real interactive terminal rendering, mouse, resize, PTY behavior, or clean exit in a user terminal. Linux ARM64 published-package smoke should be added only after a release ships `architect-mcp-tui-linux-arm64.tar.gz`.

## Post-Release Evidence

Latest recorded release: `v0.2.1`, published on 2026-05-16.

| Evidence | Platform coverage | Status | Link or note |
| --- | --- | --- | --- |
| npm publish workflow for `v0.2.1` | Ubuntu publish runner | Passed | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25968887248 |
| TUI install-smoke workflow | Ubuntu x64, Ubuntu ARM64, macOS 14, Windows latest | Passed on latest recorded PR run for pre-ARM matrix | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25970418642 |
| TUI live-QA smoke workflow | Ubuntu, macOS 14, Windows latest | Passed on latest recorded PR run | https://github.com/tonycdr-prog/architect-mcp/actions/runs/25969825040 |
| TUI published package smoke workflow | Ubuntu and Windows hosted runners | Pending first run | Hosted non-interactive baseline only; does not replace #136 manual terminal QA |
| CI terminal-evidence summaries | Ubuntu and Windows hosted runners | Baseline only | Emitted in the TUI live-QA workflow step summary; does not replace #136 manual terminal QA |
| Manual macOS source-checkout smoke | macOS local terminal | Record in the release PR or release notes when run | Maintainer evidence required before a TUI production go decision |
| Manual Linux terminal smoke | Linux local or VM terminal | Pending unless explicitly waived | Hosted workflow evidence is not a substitute for a real terminal smoke |
| Manual Windows terminal smoke | Windows Terminal or PowerShell | Pending unless explicitly waived | Hosted workflow evidence is not a substitute for a real terminal smoke |

Do not convert pending manual rows to passed status without platform, command, version, and failure-note evidence.

When Linux or Windows manual evidence is posted publicly, prefer the generated evidence command:

```bash
architect-mcp-tui terminal-evidence --markdown
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
```

When Linux and Windows reports are generated separately, pass both files to the launch judge:

```bash
architect-mcp-tui launch-judge --json --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui launch-judge --public-summary --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui collect-terminal-evidence --json --repo tonycdr-prog/architect-mcp --issue 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --public-summary --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown-output .architect-mcp/release/evidence-index.md --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --json --require-go --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136 --waive-blocker 136="maintainer accepted launch with platform QA waiver" --waive-terminal-evidence 136="maintainer accepted launch without manual Linux/Windows terminal evidence"
```

Use `launch-judge --public-summary` or `launch-readiness --public-summary` for PR comments, release notes, and issue follow-up. These summaries keep the launch decision, next actions, stack/evidence status, source filenames or platform evidence where relevant, terminal-evidence status, environment provenance, collected dates, and waiver state while omitting workspace paths, raw command tails, smoke internals, full PR/check payloads, terminal-evidence freeform source text, command summaries, notes, cache paths, private repo names, and token-shaped values. Keep the full `--json` report local unless a maintainer asks for a redacted excerpt.

Use `collect-terminal-evidence` when Linux and Windows evidence is posted as fenced JSON in a GitHub issue. It performs a read-only GitHub CLI lookup, extracts public-safe evidence blocks, applies the same launch-judge validation rules, and prints merged evidence for local release judging. It does not replace the underlying terminal QA or mutate the issue.

Use `launch-readiness` when maintainers need one public-safe report for both the PR stack and the terminal-evidence issue. It reuses `launch-stack` and `collect-terminal-evidence`, stays read-only, can discover a stacked PR chain with `--stack-from-pr`, and keeps explicit blocker waivers separate from real Linux/Windows terminal evidence. Clean terminal evidence must come from `local_terminal` or `vm_or_cloud_terminal`; hosted CI, container, unknown, or missing provenance remains conditional. If maintainers intentionally launch without real manual Linux/Windows terminal evidence, they must pass `--waive-terminal-evidence ISSUE=reason`; the report records the waiver while leaving the terminal-evidence issue section visibly incomplete.

Use `evidence-index --json` when automation needs one public-safe release artifact that includes both launch-readiness state and governance-audit state. Use `evidence-index --markdown` when maintainers need a human-readable PR comment, release-note block, or goal-ledger update. Use `evidence-index --markdown-output <path>` when that handoff should survive as a workspace artifact; the path must be relative to the workspace, parent directories are created, and no GitHub comment or release is posted automatically. Add `evidence-index --require-go` when a script should fail unless the combined evidence is exactly `go`; without that flag, `conditional_go` remains a successful handoff result so incomplete evidence can still be published honestly. These modes emit or write the combined judge result, section summaries, terminal-evidence status and environment provenance, and next actions, and should be preferred over pasting raw local `launch-readiness --json` or `governance-audit --json` output into public issues.

Unchanged issue-template placeholders are not acceptable evidence. If a report still contains `REPLACE with ...`, `issue #136 public-safe terminal QA report`, or the default placeholder command summary, the launch judge keeps the external evidence check at `conditional_go` and asks for real platform-specific QA evidence.

When launch readiness depends on a stack of open PRs plus external blocker issues, collect stack state separately:

```bash
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --pr 150 --pr 178 --blocker 136
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --required-check verify
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --pr 150 --pr 178 --blocker 136 --waive-blocker 136="maintainer accepted a temporary platform waiver with reason"
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --public-summary --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136 --waive-blocker 136="maintainer accepted a temporary platform waiver with reason" --waive-terminal-evidence 136="maintainer accepted launch without manual Linux/Windows terminal evidence"
```

`launch-stack` is a read-only GitHub CLI summary for PR and issue numbers. It can use explicit `--pr` flags or discover a stacked chain with `--stack-from-pr <number>`, ordered from the base PR to the head PR. Maintainers can repeat `--required-check <name>` when a specific CI job must exist on every listed PR; absent required checks are `no_go`. It is `no_go` for failed checks, missing required checks, dirty/unknown merge states, requested PR changes, or review-thread lookup failures; `conditional_go` for draft PRs, required review, unknown review decisions, unresolved review threads, pending checks, unstable merge states without explicit required-check evidence, or open blocker issues; and `go` only when supplied PRs are non-draft, review-compatible, have no unresolved review threads, are green, contain every required check, and supplied blocker issues are closed or explicitly waived. A GitHub `UNSTABLE` merge state can be treated as ready only when GitHub also reports `mergeable=MERGEABLE`, no checks are failed or pending, and the caller supplied at least one explicit required check that is present and green. Waivers must be supplied by the maintainer as `--waive-blocker ISSUE=reason`; the report records `status: waived` and the public reason so a waiver is visible and does not pretend evidence exists. Launch-stack JSON now reports `schemaVersion: 2` to signal this additive waiver shape.

If a maintainer must normalize already-posted notes manually, keep the same launch judge terminal evidence schema:

```json
{
  "schemaVersion": 1,
  "reports": [
    {
      "platform": "linux",
      "status": "passed",
      "environment": "local_terminal",
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
