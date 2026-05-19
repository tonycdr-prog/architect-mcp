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

Receipts help reviewers separate claimed verification text from attached execution evidence. They do not replace CI, terminal QA, or human review, and public summaries should omit raw logs, secrets, token-shaped values, and local paths.

## Optional MCP Integration Gate

When a project brief implies external tooling, use the advanced MCP catalog after the core flow has clarified the provider and boundary. `recommend_mcp_servers` should ask when the brief only says "database" or "payments". `create_mcp_install_plan` and `review_mcp_install_plan` keep installation dry-run and reviewable before any local config write.

For a concrete repository-creation walkthrough, see [New App Through The Work Gate](./new-app-work-gate.md).
