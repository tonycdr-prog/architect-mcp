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
```

Use `--skip-mcp` only when you need static evidence without spawning architect-mcp:

```bash
architect-mcp-tui governance-audit --json --skip-mcp
```

The command exits non-zero when deterministic governance checks fail. MCP review unavailability or repo-structure warnings are reported as warnings so maintainers can still inspect the report.

For an explicit launch readiness decision, use:

```bash
architect-mcp-tui launch-judge --json
architect-mcp-tui launch-judge --json --run-release-check --require-clean-git
```

`launch-judge` wraps the governance audit with terminal smoke, release-gate execution state, git worktree state, and known manual-evidence gaps. It reports `go`, `conditional_go`, or `no_go`. Missing release-gate execution, skipped smoke, skipped MCP review, adapter warnings, dirty git state without `--require-clean-git`, or uncollected Windows/Linux terminal evidence keep the result at `conditional_go`. Governance failures, failed terminal smoke, failed release gate, or a dirty git state with `--require-clean-git` produce `no_go`.

## Recurring Workflow

`.github/workflows/governance-audit.yml` runs on manual dispatch and a weekly schedule. It builds the source TUI, runs:

```bash
node bin/architect-mcp-tui.cjs governance-audit --json > governance-audit.json
```

Then it writes a public-safe GitHub step summary with status, read-only state, MCP gate status, counts, categories with findings, deterministic gates, and smoke evidence. It does not upload the raw JSON by default.

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
