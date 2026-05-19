# Governance Audit

`architect-mcp-tui governance-audit --json` is the read-only governance and drift check for maintained repositories. It is meant for existing projects after initial creation, not for mutating code or applying fixes.

## What It Checks

- Agent instructions: `AGENTS.md` exists and carries the memory policy.
- Architecture docs: `docs/architecture-contract.md` and `docs/build-plan.md`.
- Public docs: `README.md` and `llms.txt`.
- Environment hygiene: `.env.example` exists and secret-shaped local config is flagged.
- Dependency hygiene: lockfiles and `.github/dependabot.yml`.
- CI and release gates: package scripts, CI workflow, npm publish release gate, and TUI QA workflow.
- Drift evidence: live `review_local_workspace` in `mode=audit` when architect-mcp is available.
- Memory safety: durable context proposals only, filtered for secrets, raw chat, customer data, transient task details, speculative guesses, and sensitive security findings.

Memory proposals are evidence-driven and scoped to the audited workspace. The audit does not invent `architect-mcp` project memory when it is pointed at an unrelated repository; it emits proposals only when repo-local artifacts such as `package.json`, `AGENTS.md`, `docs/goal-ai-software-foundry.md`, or `docs/architecture-contract.md` support them.

## Local Command

```bash
architect-mcp-tui governance-audit --json
architect-mcp-tui governance-audit --public-summary
```

Use `--skip-mcp` only when you need static evidence without spawning architect-mcp:

```bash
architect-mcp-tui governance-audit --json --skip-mcp
architect-mcp-tui governance-audit --public-summary --skip-mcp
```

The command exits non-zero when deterministic governance checks fail. MCP review unavailability or repo-structure warnings are reported as warnings so maintainers can still inspect the report.

Use `governance-audit --public-summary` for issue comments, release notes, and external maintainer handoff. It keeps status, read-only state, category status, deterministic and smoke evidence names/counts, memory proposal safety counts, MCP review counters, finding counts, redacted findings, and next actions. It omits the workspace path, raw command strings, raw memory proposal text, raw MCP detail payloads, finding evidence fields, private names, local paths, and token-shaped values. Keep the full `--json` report local unless a maintainer asks for a redacted excerpt.

For an explicit launch readiness decision, use:

```bash
architect-mcp-tui launch-judge --json
architect-mcp-tui launch-judge --json --run-release-check --require-clean-git
architect-mcp-tui launch-judge --json --terminal-evidence terminal-evidence.json
architect-mcp-tui launch-judge --json --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui launch-judge --public-summary --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --pr 150 --blocker 136
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --public-summary --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown-output .architect-mcp/release/evidence-index.md --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --json --require-go --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136 --waive-blocker 136="maintainer accepted launch with platform QA waiver" --waive-terminal-evidence 136="maintainer accepted launch without manual Linux/Windows terminal evidence"
```

`launch-judge` wraps the governance audit with terminal smoke, release-gate execution state, git worktree state, and external terminal evidence. It reports `go`, `conditional_go`, or `no_go`. Missing release-gate execution, skipped smoke, skipped MCP review, adapter warnings, dirty git state without `--require-clean-git`, or missing Linux/Windows terminal evidence keep the result at `conditional_go`. Governance failures, failed terminal smoke, failed release gate, unsafe terminal evidence, failed external terminal QA, or a dirty git state with `--require-clean-git` produce `no_go`.

`launch-stack` is the companion check for explicit or discovered PR stacks and external blocker issues. It uses GitHub CLI read-only lookups and reports `no_go` for failed checks, missing explicit `--required-check` names, dirty/unknown merge states, or requested PR changes; `conditional_go` for draft PRs, required review, unknown review decisions, pending checks, temporarily unstable merge states caused by pending checks, or open blockers; and `go` only when the supplied PRs are clean, non-draft, green, review-compatible, contain every explicitly required check, and supplied blockers are closed. Use `--stack-from-pr <number>` to follow a stacked PR chain from the head PR back to the first non-PR base branch.

`launch-readiness` is the maintainer rollup when both PR-stack state and public terminal-evidence issue state matter. It reuses the same read-only GitHub CLI lookups, including any repeated `--required-check <name>` inputs, reports a combined decision, and keeps explicit blocker waivers distinct from real terminal evidence. `--waive-terminal-evidence ISSUE=reason` records a public maintainer decision for missing or incomplete manual terminal evidence; it does not override malformed or unsafe evidence.

Use `--public-summary` when posting governance-audit, launch-judge, or launch-readiness evidence publicly. The public summaries keep the governance or launch decision, check or stack summaries, missing required-check evidence, next actions, release-gate attempt/result where relevant, terminal-evidence source filenames or platform status, and waiver state, but omit the workspace path, full governance audit, full smoke report, full PR/check payloads, raw memory proposal text, terminal-evidence freeform source text, command summaries, notes, stdout/stderr tails, cache paths, private repo names, and token-shaped values.

Use `evidence-index --json` for machine-readable release handoff when a maintainer needs one public-safe artifact that includes both the launch-readiness public summary and the governance-audit public summary. Use `evidence-index --markdown` for release notes, PR comments, and goal-ledger handoff. Use `evidence-index --markdown-output <path>` to keep the same public-safe Markdown as a relative workspace artifact without posting to GitHub automatically. Use `evidence-index --require-go` for strict launch automation that should fail on `conditional_go` instead of merely printing the next actions. The index reports one combined judge result and section-level summaries while preserving waiver visibility and keeping raw local diagnostics out of public text.

`--terminal-evidence` is for short, public-safe Linux and Windows QA summaries, not raw output. Pass it more than once when each platform report is saved separately. The JSON schema is:

```json
{
  "schemaVersion": 1,
  "reports": [
    {
      "platform": "linux",
      "status": "passed",
      "environment": "local_terminal",
      "source": "issue #136 public-safe summary",
      "commandSummary": "architect-mcp-tui help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "summary only, no raw logs"
    },
    {
      "platform": "windows",
      "status": "passed",
      "environment": "vm_or_cloud_terminal",
      "source": "issue #136 public-safe summary",
      "commandSummary": "architect-mcp-tui help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "summary only, no raw logs"
    }
  ]
}
```

Do not include secrets, raw stdout or stderr, private repo names, absolute local paths, customer data, or sensitive security details. The command fails closed when the evidence contains raw log fields, local paths, secret-shaped strings, failed platform status, duplicate platforms, or unsupported platforms.

## Recurring Workflow

`.github/workflows/governance-audit.yml` runs on manual dispatch and a weekly schedule. It builds the source TUI, runs:

```bash
node bin/architect-mcp-tui.cjs governance-audit --public-summary > governance-audit-public-summary.json
```

Then it writes a GitHub step summary from that public summary with status, read-only state, MCP gate status, counts, categories with findings, deterministic gate names, and smoke evidence names. If the audit fails, the workflow still writes the public-safe summary first, then fails the job afterward. It does not upload the raw JSON by default.

## Public-Safe Issue Evidence

Use the `Governance audit report` issue form when sharing results. Do not paste:

- Secrets, tokens, or credentials.
- Private repository names.
- Absolute local paths.
- Raw file contents.
- Raw audit JSON.
- Raw conversation logs.
- Customer data.
- Sensitive security details.

The useful public fields are:

- Status: `passed`, `passed_with_warnings`, or `failed`.
- Read-only state.
- MCP gate status.
- Files reviewed.
- Error and warning counts.
- Categories with findings.
- Deterministic gates, especially `npm run release:check`.
- Smoke evidence commands.

For repositories that have not adopted architect-mcp governance artifacts, a failed audit can still be useful read-only evidence. Treat missing contracts, build plans, agent instructions, release gates, or env templates as adoption findings, not as proof that the repository was mutated.

## Release Gate

The audit is evidence, not the release gate. `npm run release:check` remains the clean-checkout release-sensitive gate.
