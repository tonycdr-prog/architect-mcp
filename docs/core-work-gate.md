# Core Work Gate

The core work gate is the default product story. It is deliberately smaller than the full advanced surface so agents start with the behavior that matters most: clarify, constrain, review, verify.

Boundary note: the MCP tools are report-only unless a client, host, TUI session, CI workflow, or human reviewer enforces the result. For untrusted input handling and gate-bypass cases, see [Prompt Injection And Gate Bypass Threat Model](./prompt-injection-threat-model.md).

## Tool Sequence

1. `grill_me`: pressure-test the brief and stop while blockers remain.
2. `create_pre_edit_contract`: capture intent, likely files, non-goals, assumptions, and verification checks before risky edits.
3. `review_build_plan`: ensure implementation slices have inputs, outputs, allowed directories, forbidden files, checks, and stop conditions.
4. `review_proposed_file_plan`: catch monolithic or boundary-breaking file plans before files are written.
5. `review_repo_structure`: review file summaries and directories, including hosted-safe reviews.
6. `review_implementation_against_contract`: detect drift, skipped verification, or widened scope.
7. `review_agent_final_response`: check that the final message states changed files, verification, assumptions, remaining work, and evidence honestly.
8. `review_agent_session`: combine intent, contract, changed files, verification, memory, and final response into one report.

## Pre-Edit Contract

A useful contract should state:

- Intended behavior and user-visible scope.
- Likely files and non-goals.
- Accepted assumptions.
- Verification commands the agent is allowed to claim.
- Stop conditions for uncertainty, security boundaries, and expanded scope.

## Build Plan Review

Build-plan slices are contracts. Each slice should have a small goal, explicit ownership boundaries, exact checks, and a stop condition. The review flags slices that are too broad, invent verification commands, or miss `AGENTS.md`, `docs/architecture-contract.md`, or `docs/build-plan.md` when creating a repo.

## Implementation Review

After edits, provide changed file summaries and verification results. The implementation review should fail if the agent changed files outside the contract, skipped required checks, or expanded behavior without a new decision.

## Final Response Review

The final response should be specific and evidence-backed:

- What changed.
- Which checks passed, failed, were skipped, or were not run.
- Structured command receipts when the client has them: command, status, source, timestamp or run id, and a public-safe summary.
- Assumptions and remaining gaps.
- No unsupported root-cause claims.

Receipts help reviewers separate claimed verification text from attached execution evidence. The review output treats required-check wording as claimed evidence, fresh session records and local/TUI/adapter/manual receipts as supplied evidence, CI receipts as independently resolvable evidence only when they include a usable `recordedAt` timestamp or `runId`, and receipts with missing, stale, future, or invalid freshness as unverifiable provenance. Bare CI receipt summaries without either handle are unverifiable rather than fresh supplied proof. Local, TUI, adapter, and manual receipts need a fresh `recordedAt` before they count as fresh matching evidence; a local `runId` alone is not independent proof. Receipts do not replace CI, terminal QA, release gates, or human review, and public summaries should omit raw logs, secrets, token-shaped values, and local paths.

## Direct-Client Sequence Receipts

Direct MCP clients can attach `create_work_gate_sequence_receipt` output when they need to show more than one favorable review result. A sequence receipt is public-safe and request-scoped: it records required gate order, pass/warn/fail status, whether each gate had its required inputs and review evidence, and timestamp or run-id presence without storing raw payloads.

Use it with `audit_work_gate_completeness` when a PR or launch judge needs to distinguish a complete direct-client work-gate run from a partial set of MCP calls. It is still detection-only; clients, CI, TUI sessions, reviewers, or humans must enforce the result.

## Optional MCP Integration Gate

When a project brief implies external tooling, use the advanced MCP catalog after the core flow has clarified the provider and boundary. `recommend_mcp_servers` should ask when the brief only says "database" or "payments". `create_mcp_install_plan` and `review_mcp_install_plan` keep installation dry-run and reviewable before any local config write.

For a concrete repository-creation walkthrough, see [New App Through The Work Gate](./new-app-work-gate.md).
