# Advanced Standards-Intelligence Criteria

Compatibility note: this file keeps the historical `docs/v5-scope.md` path and V5-linked internal identifiers for scripts, tools, and tests. Public product language should describe this as an advanced standards-intelligence maturity layer, not as a product version.

This criteria layer remains local-first. It excludes hosted service work, database/storage work, GitHub/PR adapters, CLI UX, and typed SDKs.

It should make architect-mcp more self-improving, more explainable, and easier for agents to apply correctly inside the current MCP tool surface.

## Standards-Intelligence Direction

This layer focuses on local intelligence and usability of the standards harness:

- Standards orchestration.
- Explainable findings.
- Policy simulation.
- Cross-stack conflict handling.
- Eval expansion.
- Agent teaching and handoff quality.
- Repo-profile intelligence.
- Contract lifecycle quality.

## Standards Orchestration

Goal: make standards selection smarter without adding storage or network dependencies.

Deliverables:

- A standards resolver that explains why each foundation, policy, and stack pack was selected.
- Conflict ranking when multiple packs apply to the same files or concerns.
- A "minimum viable standards" mode for small projects.
- A "strict production standards" mode for high-risk projects.
- A standards coverage report by project brief, stack, and file summary signals.

## Explainable Findings

Goal: make review findings easier for novice users and agents to act on.

Deliverables:

- Plain-English finding explanations.
- "What this probably means" sections for common terms like auth, API boundary, migration, dependency risk, and accessibility.
- Suggested next tool calls for each finding.
- Fix-shape recommendations: split file, add boundary, add test, ask user, verify, or defer.
- Evidence snippets showing which file signals triggered each finding.

## Policy Simulation

Goal: let clients preview how stricter or looser policy settings would affect a repo before enforcing them.

Deliverables:

- Simulate review gates across `summary`, `migration`, `ci`, and custom thresholds.
- Show which findings would become blockers under stricter modes.
- Show which baselines or accepted findings would stop suppressing issues.
- Preview impact of adding a new policy pack or stack pack.

## Cross-Stack Conflict Handling

Goal: improve behavior when multiple stack packs give overlapping or competing advice.

Deliverables:

- Better conflict explanations.
- Conflict severity ranking.
- Suggested resolution: prefer framework pack, prefer policy pack, ask user, or split path scopes.
- Tests for overlapping auth, validation, frontend, backend, and AI/tool rules.

## Eval Expansion

Goal: make the harness harder to regress without relying on live services.

Deliverables:

- Standards-intelligence eval suites for novice-intent interpretation, standards selection, explainable findings, policy simulation, conflict resolution, and handoff quality.
- Golden fixture reports for common project archetypes.
- Regression fixtures for known agent failure modes.
- Eval summaries that explain failures in product language, not just assertion messages.

## Agent Teaching

Goal: help agents and novice users understand what to do next without turning findings into noisy lectures.

Deliverables:

- Concise teaching snippets attached to high-value findings.
- Terminology mapping for common novice phrases.
- One-question clarification suggestions for ambiguous fixes.
- "Do this / avoid this / prove it with this" output blocks.
- Agent handoff summaries that are short enough to reuse in the next turn.

## Repo Profile Intelligence

Goal: make existing repo layout handling stronger without adding external integrations.

Deliverables:

- More built-in repo profiles.
- Profile confidence scoring from file summaries.
- Profile mismatch warnings when a contract forces the wrong structure.
- Suggested `repoLayout.pathMap` updates.
- Fixtures for monorepos, packages/apps layouts, mobile apps, docs-heavy repos, and mixed frontend/backend repos.

## Contract Lifecycle

Goal: make architecture contracts easier to evolve safely.

Deliverables:

- Contract maturity levels.
- Contract diff explanations in plain English.
- Safer rule deprecation and replacement guidance.
- Contract changelog generation from diffs.
- Better warnings when new rules would create too many findings for an existing repo.

## Non-Goals

- Hosted service or hosted API implementation.
- Database/storage, including schema, persistence, migrations, review history, or memory storage.
- GitHub repository, branch, PR, issue, app, OAuth, or comment workflow.
- CLI UX.
- Typed client SDK.
- Billing, accounts, teams, or dashboards.

Those remain out of scope until a later backend/productization phase.
