# Advanced Operating-Model Criteria

Compatibility note: this file keeps the historical `docs/v9-scope.md` path and V9-linked internal identifiers for scripts, tools, and tests. Public product language should describe this as an advanced operating-model eval layer, not as a product version.

This criteria layer remains local-first. It excludes hosted service work, database/storage work, GitHub/PR adapters, CLI UX, and typed SDKs.

It should make architect-mcp easier for agents and clients to run correctly by composing the advanced standards-intelligence, governance, strategic-planning, and automation capabilities into deterministic local workflows with normalized outputs, token-aware summaries, and scenario-level acceptance criteria.

## Operating-Model Direction

This layer focuses on local operating-model readiness:

- Local orchestration recipes.
- Scenario acceptance profiles.
- Result normalization.
- Context-budget governance.
- Evidence routing.
- Source provenance continuity.
- Advisory skill-pattern routing.
- Artifact-quality routing.
- Local dry-run plans.
- Tool-loop quality gates.

## Local Orchestration Recipes

Goal: define deterministic local tool sequences for common work without adding a CLI, hosted service, or external coordinator.

Deliverables:

- Recipes for fresh app planning, existing repo review, vague bug fix, risky refactor, stack-pack promotion, documentation refresh, and standards calibration.
- Each recipe lists required inputs, recommended MCP tools, decision gates, expected outputs, and proof requirements.
- Recipe selection from request text, project brief, file summaries, and harness risk.
- Warnings when a client skips a required pre-edit, review, or verification step.
- Tests that recipes stay local-only and avoid productization dependencies.

## Scenario Acceptance Profiles

Goal: judge whether a complete local run is good enough for the user’s scenario, not only whether individual tools returned valid data.

Deliverables:

- Acceptance profiles for fresh apps, feature work, bug fixes, UI polish, security-sensitive changes, dependency/config changes, documentation updates, and standards-pack authoring.
- Profile-specific pass/fail checks for intent clarity, contract coverage, file-boundary safety, evidence, verification, and final-response honesty.
- Plain-English failure summaries for novice users.
- Machine-readable acceptance status for agents.
- Fixture runs that combine multiple MCP outputs into one scenario verdict.

## Result Normalization

Goal: reduce agent confusion by giving cross-tool outputs a consistent structure.

Deliverables:

- Shared local result envelope for status, stoplight, findings, evidence, assumptions, next actions, proof, warnings, and handoff.
- Finding normalization across repo review, harness review, artifact review, security review, stack-pack review, and scenario acceptance.
- Stable severity, confidence, and action categories.
- Normalized "not done" and "could not verify" fields.
- Tests for backwards-compatible response shapes.

## Context-Budget Governance

Goal: keep the MCP useful in YOLO-style agent loops without flooding context.

Deliverables:

- Output modes: `compact`, `standard`, and `full`.
- Evidence budgets by finding severity and scenario.
- Summary precedence rules: blockers first, then assumptions, then proof, then optional education.
- Token-sensitive guidance compression for stack packs, playbooks, and policy bundles.
- Warnings when a requested output would exceed a supplied context budget.

## Evidence Routing

Goal: make evidence easier to connect to decisions, findings, verification, and final responses.

Deliverables:

- Evidence IDs that can be referenced across local tool outputs.
- Required evidence classes by scenario and blast radius.
- Evidence-to-finding and evidence-to-verification mapping.
- Root-cause claim guardrails that require linked evidence.
- Fixtures for missing, weak, conflicting, and sufficient evidence.

## Source Provenance Continuity

Goal: keep source-backed standards auditable after they move through orchestration, playbooks, scenario reports, and handoff summaries.

Deliverables:

- Preserve `llms.txt` source identity, snapshot path, fetched timestamp, hash, and evidence references where available.
- Show when a rule came from a source-backed stack pack, foundation pack, policy pack, built-in heuristic, or client-supplied rule.
- Mark missing source snapshots as warnings, not blockers, unless a scenario explicitly requires source-backed evidence.
- Include source provenance in normalized findings and compact handoff summaries when it affects trust or remediation.
- Tests that source-backed rules do not become opaque inside recipes, scenario acceptance, or result normalization.

## Advisory Skill-Pattern Routing

Goal: use the built-in skills catalog as source material for better local workflow selection without requiring users to have local Codex skills installed.

Deliverables:

- Recipe selection can recommend advisory skill patterns such as MCP config security, verification-before-completion, agent instruction quality, eval discipline, and memory scope conventions.
- External client-supplied skill metadata remains advisory only and never overrides the user request, contract, or current repo evidence.
- Skill recommendations include reason, confidence, expected value, and why they are safe to ignore.
- Unsafe skill metadata is surfaced as a warning, not executed.
- Tests for built-in catalog recommendations and client-supplied skill metadata.

## Artifact-Quality Routing

Goal: make agent-facing docs part of the local operating model instead of a separate afterthought.

Deliverables:

- Scenario acceptance profiles can require checks for `AGENTS.md`, `llms.txt`, generated agent instructions, setup commands, test commands, build commands, repo boundaries, verification rules, and security notes.
- Tool-loop quality gates warn when implementation changes make agent instructions or navigation docs stale.
- Documentation refresh recipes include artifact scoring, missing-example detection, and concise update suggestions.
- Result normalization includes artifact findings alongside code, harness, stack-pack, and security findings.
- Tests for stale, missing, weak, and passing artifact guidance.

## Local Dry-Run Plans

Goal: let agents preview the full local governance run before editing code.

Deliverables:

- Dry-run plan output that shows likely recipe, required inputs, gates, selected standards, verification checks, and expected final response contract.
- No filesystem writes required by the MCP; clients decide whether to persist outputs.
- Difference between "ready to edit", "confirm first", and "blocked until clarified".
- Risk notes for broad file plans, dependency/config changes, destructive commands, and public API changes.
- Source-backed guidance, advisory skill patterns, and artifact checks expected for the scenario.
- Tests for vague novice prompts and concrete low-risk prompts.

## Tool-Loop Quality Gates

Goal: detect when an agent is using the MCP poorly even if individual tool calls succeed.

Deliverables:

- Detect skipped interpretation before risky edits.
- Detect missing pre-edit contract before broad changes.
- Detect review without verification.
- Detect final responses that omit changed, verified, assumptions, or not-done sections.
- Return corrective next-step guidance that is short enough to paste into an agent turn.

## Non-Goals

- Hosted service or hosted API implementation.
- Database/storage, including schema, persistence, migrations, review history, or memory storage.
- GitHub repository, branch, PR, issue, app, OAuth, or comment workflow.
- CLI UX.
- Typed client SDK.
- Billing, accounts, teams, dashboards, or remote policy management.

Those remain out of scope until a later backend/productization phase.
