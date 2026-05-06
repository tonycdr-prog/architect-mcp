# V4 Scope Audit

Audit date: 2026-05-05

## Verdict

V4 is allowed to proceed only as local-first standards depth and agent workflow work. Anything hosted, database/storage, GitHub, PR-adapter, CLI, or SDK related is out of scope until the backend is concrete.

## Findings

### Fixed: V4 Included PR/GitHub-Adjacent Workflow

Previous V4 wording included `PR Review` and user-owned GitHub/PR memory recipes. That pulled GitHub-shaped adoption work into V4 too early.

Resolution:

- Renamed `PR Review` to `Change Review`.
- Removed GitHub/PR adapter recipes from memory proposals.
- Kept the core capability as storage-neutral changed-file review.

### Fixed: V4 Included Database/Migration Work

Previous V4 wording included slow database query patterns, migrations, schema ownership, data movement, and migration tests. That belongs after backend/data architecture exists.

Resolution:

- Removed database query and migration work from V4.
- Kept only local, file-summary-friendly standards: security, accessibility, performance, testing, dependency hygiene, verification honesty, and handoff quality.

### Fixed: V4 Non-Goals Were Too Narrow

Previous non-goals mentioned hosted service, database persistence, and GitHub memory adapter, but left room for related design work to slip in.

Resolution:

- Expanded non-goals to cover hosted API/auth/billing/dashboard, database/storage/schema/migrations/review history/memory storage, and GitHub/PR adapters.

## Approved V4 Scope

- Source-backed local policy packs.
- Security rules that can be evaluated from file summaries/local scans/config inputs.
- Accessibility rules that can be evaluated from frontend file summaries, stack packs, or generated artifacts.
- Performance rules that avoid backend/database assumptions.
- Testing and verification honesty.
- Dependency hygiene.
- Stronger `review_agent_session`.
- Storage-neutral memory proposal quality.
- Change review over supplied changed-file summaries.
- Agent handoff summaries.
- YOLO budget counters.
- V4 eval suites.

## Deferred Until Backend/Productization Phase

- Hosted service.
- Hosted API implementation.
- Hosted dashboard.
- Hosted auth/billing/accounts.
- Database/storage, including schema, persistence, migrations, review history, and memory storage.
- GitHub repository, branch, PR, issue, or app adapters.
- CLI UX.
- Typed client SDK.

These are not V5 requirements. V5 and later local-first roadmap phases can continue without them while the MCP core becomes more concrete.
