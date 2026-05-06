# V4 Scope

V4 should build on the complete local-first V1-V3 core. It must not include hosted service work, database/storage work, GitHub/PR adapters, CLI UX, or a typed SDK. Those wait until the backend shape is concrete.

V4 stays inside the existing MCP/domain architecture: richer standards, richer file-summary/local-scan review, stronger agent workflow outputs, and more deterministic evals.

## V4 Standards Depth

Goal: make architect-mcp better at loading, applying, and reviewing broad engineering quality standards, not only architecture structure.

### Source-Backed Stack Packs

- Promote more ingested `llms.txt` sources into versioned packs only when they produce concrete, reviewable rules.
- Prioritize security, accessibility, performance, testing, dependency hygiene, agent instruction quality, verification honesty, and framework-specific production readiness.
- Require every promoted rule to include source provenance, detector mapping, examples, verification expectations, and conflict review.
- Keep live fetching explicit; default to local ingested snapshots.

### Broader Quality Rules

Security:

- Secret handling, auth/session boundaries, authorization checks, webhook verification, SSRF/path traversal risk, MCP config safety, and dependency risk.

Accessibility:

- Semantic UI structure, keyboard flows, form labeling, focus handling, color/contrast policy, and mobile/touch ergonomics for frontend stack packs.

Performance:

- Large render surfaces, unnecessary client/server boundaries, heavyweight dependencies, bundle-risk hints, and avoidable repeated work.

Testing:

- Test proximity, behavior coverage, regression proof, eval fixture coverage, mutation/migration tests, and verification-command alignment.

Dependency Hygiene:

- Upgrade reason, compatibility risk, lockfile impact, rollback plan, audit state, package-surface changes, and dependency-boundary drift.

### V4 Standards Deliverables

- Versioned V4 policy/foundation packs for security, accessibility, performance, testing, dependency hygiene, verification honesty, and agent handoff quality.
- More executable detector families where signals are available from file summaries or local scans.
- Review findings with stable codes, confidence, and remediation text.
- Fixture coverage for clean and failing examples per policy area.
- Stack-pack quality matrix that distinguishes source depth, executable detector coverage, docs, and tests.

## V4 Agent Workflow

Goal: improve how clients run coding agents through a whole work session, from vague intent to final proof, without slowing low-risk work.

### Harness Loops

- Strengthen `review_agent_session` into the primary end-of-work gate for clients.
- Add richer session summaries that capture changed scope, assumptions, triggered standards, skipped checks, unresolved risks, and next-agent handoff.
- Detect intent debt when too many assumptions or yellow decisions accumulate across a session.
- Make confidence-cliff downgrades explicit after repo inspection changes the original risk picture.

### Change Review

- Define a change-review input shape using changed file summaries, intended contract, verification output, and optional baseline.
- Compare changes against pre-edit contracts, stack packs, policy packs, and generated agent instructions.
- Flag unrelated files, scope upgrades, missing tests, skipped checks, dependency churn, and public API drift.

### Memory Proposals

- Keep memory stateless in core V4.
- Improve proposal quality: scope, sensitivity, expiry/staleness, invalidation condition, source summary, and token-cost estimate.
- Add memory-review recipes that stay storage-neutral and return structured proposals only.

### Agent Handoff

- Generate concise handoff blocks after intent confirmation, after implementation review, and after final-response review.
- Include current contract, loaded standards, assumptions, verification status, remaining risks, and next recommended tool call.

### YOLO Budgets

- Formalize `strict`, `guided-yolo`, and `full-yolo` as client-facing policy profiles.
- Add budget counters for assumptions, skipped checks, widened files, risky terms, and unverified claims.
- Let low-risk reversible changes proceed quickly while medium/high-risk work asks one focused question.

### Policy Packs

- Treat policy packs as composable standards applied before stack packs.
- Keep policy packs local-first and versioned like foundation/stack packs.
- Candidate V4 policy packs: security, accessibility, performance, testing, dependency hygiene, verification honesty, and agent handoff quality.

### V4 Workflow Deliverables

- Stronger `review_agent_session` coverage and examples.
- Change-review recipe and fixtures.
- Expanded memory proposal/relevance tests.
- YOLO budget outputs and tests.
- Handoff summary output contract.
- V4 eval suites for standards depth and agent workflow.

## Non-Goals

- Hosted service.
- Database/storage work, including schema, persistence, migrations, review history, or memory storage.
- GitHub or PR adapter implementation.
- Hosted API, hosted policy, hosted auth, hosted billing, or hosted dashboard work.
- CLI UX.
- Typed client SDK.

Those stay deferred until a later backend/productization phase after the local MCP architecture is concrete.
