# V7 Scope

V7 is the next local-first maturity layer after V6. It still excludes hosted service work, database/storage work, GitHub/PR adapters, CLI UX, and typed SDKs.

V7 should make architect-mcp better at strategic planning, local simulation, standards negotiation, and self-improvement while staying inside the existing MCP/domain/test architecture.

## V7 Direction

V7 focuses on local strategic intelligence:

- Architecture strategy maps.
- Standards negotiation.
- Local what-if planning.
- Agent behavior diagnostics.
- Rule authoring assistance.
- Quality trend snapshots without persistence.
- Human-readable governance packs.

## V7 Architecture Strategy Maps

Goal: help agents understand how project goals, risks, stack choices, standards, and verification connect.

Deliverables:

- Strategy-map output from a project brief and selected packs.
- Risk-to-standard mapping.
- Workflow-to-boundary mapping.
- Verification-to-risk mapping.
- Gaps where important risks have no standard, pack, or verification check.

## V7 Standards Negotiation

Goal: make tradeoffs explicit when strict standards would slow or over-constrain the project.

Deliverables:

- Compare `minimal`, `balanced`, and `strict` standards profiles.
- Explain what each profile catches and what it allows.
- Suggest one focused question when standards conflict with user speed preferences.
- Mark non-negotiable safety rules separately from adjustable quality rules.

## V7 Local What-If Planning

Goal: preview likely review outcomes before an agent writes code.

Deliverables:

- What-if review for proposed file plans plus selected standards profiles.
- Impact preview for adding/removing a pack or policy bundle.
- Blast-radius forecast from proposed files, change type, and standards.
- Verification forecast: checks likely required before claiming completion.

## V7 Agent Behavior Diagnostics

Goal: identify recurring agent failure patterns from supplied session summaries and review outputs without storing history.

Deliverables:

- Diagnose patterns such as over-broad edits, skipped verification, premature root-cause claims, tool overuse, repeated assumptions, and ignored contracts.
- Return corrective steering text for the next agent turn.
- Produce compact "watch for this" handoff notes.
- Score whether the agent followed the intended harness loop.

## V7 Rule Authoring Assistance

Goal: help maintainers write better local rules without live fetching or external systems.

Deliverables:

- Draft candidate rules from supplied source text, findings, or examples.
- Explain whether a rule is executable, manual-only, too vague, duplicated, or too broad.
- Suggest detector metadata and fixture pairs.
- Suggest rule names, finding codes, examples, and remediation text.

## V7 Quality Trend Snapshots

Goal: compare two explicitly supplied review reports without persistence.

Deliverables:

- Trend comparison between before/after reports.
- New, resolved, worsened, improved, and unchanged finding groups.
- Standards coverage delta.
- Verification honesty delta.
- Suggested next local eval fixture based on recurring findings.

## V7 Human-Readable Governance Packs

Goal: produce standards packs that humans can review before agents enforce them.

Deliverables:

- Markdown rendering for policy packs and stack packs.
- Plain-English summaries of rules, triggers, examples, and evidence.
- "Adopt now / trial / reject" recommendation blocks.
- Review checklist for maintainers approving a pack.

## Non-Goals

- Hosted service or hosted API implementation.
- Database/storage, including schema, persistence, migrations, review history, or memory storage.
- GitHub repository, branch, PR, issue, app, OAuth, or comment workflow.
- CLI UX.
- Typed client SDK.
- Billing, accounts, teams, dashboards, or remote policy management.

Those remain out of scope until a later backend/productization phase.
