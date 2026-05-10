# Advanced Governance-Automation Criteria

Compatibility note: this file keeps the historical `docs/v8-scope.md` path and V8-linked internal identifiers for scripts, tools, and tests. Public product language should describe this as an advanced governance-automation maturity layer, not as a product version.

This criteria layer remains local-first. It excludes hosted service work, database/storage work, GitHub/PR adapters, CLI UX, and typed SDKs.

It should make architect-mcp better at local governance automation, standards refactoring, and resilient agent collaboration while staying inside the existing MCP/domain/test architecture.

## Governance-Automation Direction

This layer focuses on local governance automation:

- Standards refactoring.
- Policy minimization.
- Local review playbooks.
- Agent collaboration protocols.
- Failure-mode drills.
- Rule impact calibration.
- Documentation intelligence.

## Standards Refactoring

Goal: help maintainers improve existing packs and policies without changing runtime architecture.

Deliverables:

- Detect duplicate, overlapping, stale, or weak rules across packs.
- Suggest rule splits, merges, renames, and severity changes.
- Identify rules with weak examples or vague remediation.
- Suggest detector families for manual-only rules.
- Generate before/after pack diffs with risk notes.

## Policy Minimization

Goal: help projects avoid overloading agents with too many rules.

Deliverables:

- Minimal policy set recommendations for a brief and file summary.
- Rule usefulness scoring based on triggered signals and selected stack.
- Noise forecast when too many packs apply.
- "Drop, defer, or keep" recommendations for candidate rules.
- Compact standards profile output for token-sensitive clients.

## Local Review Playbooks

Goal: turn recurring review situations into repeatable local workflows.

Deliverables:

- Playbooks for fresh app generation, feature addition, refactor, security-sensitive change, dependency update, UI polish, and test hardening.
- Each playbook defines intake checks, pre-edit contract fields, relevant policy packs, review tools, and proof requirements.
- Playbook selection from request text and project brief.
- Playbook conformance review after implementation.

## Agent Collaboration Protocols

Goal: make multi-agent or multi-step local work safer without subagent orchestration, hosted state, or external systems.

Deliverables:

- Handoff protocol templates.
- File ownership declaration checks.
- Parallel-work conflict warnings from proposed file plans.
- "Do not touch" boundary declarations.
- Integration checklist for merging independently produced local changes.

## Failure-Mode Drills

Goal: test whether the harness catches known bad agent behavior before release.

Deliverables:

- Drill fixtures for skipped verification, fake root-cause claims, dependency churn, broad rewrites, misplaced secrets, UI-to-server leaks, and rule overreach.
- Drill runner output that explains what was caught and what escaped.
- Suggested new detectors or evals for escaped failures.

## Rule Impact Calibration

Goal: tune severity, confidence, and gate impact locally.

Deliverables:

- Compare severity settings across policy profiles.
- Identify rules that are too noisy or too weak for a project archetype.
- Recommend confidence adjustments from available evidence.
- Preview gate result changes from severity/confidence edits.
- Keep calibration stateless and based only on supplied reports or fixtures.

## Documentation Intelligence

Goal: keep human and agent docs aligned with actual tool behavior.

Deliverables:

- Detect stale claims in README, `llms.txt`, AGENTS.md, and generated docs.
- Summarize tool surface changes into doc update suggestions.
- Identify missing examples for new tools or rules.
- Validate that roadmap docs match tested scope boundaries.
- Generate concise release notes from supplied local change summaries.

## Non-Goals

- Hosted service or hosted API implementation.
- Database/storage, including schema, persistence, migrations, review history, or memory storage.
- GitHub repository, branch, PR, issue, app, OAuth, or comment workflow.
- CLI UX.
- Typed client SDK.
- Billing, accounts, teams, dashboards, or remote policy management.

Those remain out of scope until a later backend/productization phase.
