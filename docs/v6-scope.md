# Advanced Local-Governance Criteria

Compatibility note: this file keeps the historical `docs/v6-scope.md` path and V6-linked internal identifiers for scripts, tools, and tests. Public product language should describe this as an advanced local-governance maturity layer, not as a product version.

This criteria layer remains local-first. It excludes hosted service work, database/storage work, GitHub/PR adapters, CLI UX, and typed SDKs.

It should make architect-mcp more adaptive, auditable, and useful across longer agent workflows while staying inside the existing MCP/domain/test architecture.

## Local-Governance Direction

This layer focuses on local governance maturity:

- Policy composition.
- Standards lifecycle.
- Multi-turn agent continuity.
- Review explainability at scale.
- Local report artifacts.
- Regression intelligence.
- Cross-repo pattern portability without external integrations.

## Policy Composition

Goal: let clients assemble reusable policy sets without persistence or hosted policy management.

Deliverables:

- Policy bundle definitions as versioned local JSON files.
- Bundle validation, conflict checks, and coverage reports.
- Bundle preview against supplied file summaries.
- Recommended bundle selection from project brief and stack signals.
- Tests for strict, migration, frontend-heavy, backend-heavy, security-focused, and novice-friendly bundles.

## Standards Lifecycle

Goal: make standards evolve safely over time.

Deliverables:

- Policy and stack-pack changelog generation.
- Rule maturity levels: experimental, recommended, strict, deprecated.
- Deprecation warnings with replacement rules.
- Compatibility checks when a contract references older pack versions.
- Report whether a new ruleset is likely to create noisy findings.

## Multi-Turn Agent Continuity

Goal: improve long-running local sessions without durable storage.

Deliverables:

- Session-state summaries that clients can pass back explicitly.
- Compact continuation context for the next tool call.
- Intent-debt accumulation across supplied session summaries.
- Assumption aging within a provided session payload.
- Handoff quality scoring.

## Review Explainability At Scale

Goal: keep large review outputs understandable.

Deliverables:

- Finding clustering by root cause, affected boundary, stack, and verification gap.
- Top-priority remediation paths.
- "Fix one thing first" recommendation.
- Noise-risk scoring.
- Plain-English report summaries for novice users and concise machine-readable summaries for agents.

## Local Report Artifacts

Goal: generate portable artifacts without needing hosted dashboards or external systems.

Deliverables:

- Markdown review report output.
- JSON review report output optimized for CI or agent clients.
- Contract change report output.
- Standards coverage report output.
- Session handoff report output.

The MCP should return report contents; writing files remains a client decision unless a local-only tool already owns that behavior.

## Regression Intelligence

Goal: make known failure modes first-class.

Deliverables:

- Catalog of known agent failure patterns.
- Mapping from failure patterns to review rules and eval fixtures.
- Regression risk score for proposed changes.
- Fixture generator suggestions for new rules.
- Coverage gaps when a rule has no positive and negative fixture.

## Cross-Repo Pattern Portability

Goal: reuse lessons across repos without external memory, hosted storage, or GitHub.

Deliverables:

- Portable local pattern cards.
- Pattern-card validation and sensitivity review.
- Apply-pattern preview against a project brief or file summaries.
- Conflict checks between pattern cards and current contract.
- Token-budgeted pattern summaries.

## Non-Goals

- Hosted service or hosted API implementation.
- Database/storage, including schema, persistence, migrations, review history, or memory storage.
- GitHub repository, branch, PR, issue, app, OAuth, or comment workflow.
- CLI UX.
- Typed client SDK.
- Billing, accounts, teams, dashboards, or remote policy management.

Those remain out of scope until a later backend/productization phase.
