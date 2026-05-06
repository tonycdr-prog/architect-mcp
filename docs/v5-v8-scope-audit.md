# V5-V8 Scope Audit

Date: 2026-05-06

Scope: V5 through V8 roadmap documents. This audit checks roadmap continuity, local-first boundaries, productization drift, and whether the next version should stay away from GitHub, hosted service, and database/storage work.

## Verdict

V5 through V8 are coherent and remain local-first. They correctly avoid GitHub adapters, hosted implementation, database/storage persistence, CLI UX, typed SDKs, accounts, billing, dashboards, and remote policy management.

The main gap is not missing productization. The gap is a local operating model: V5 through V8 add many strong capabilities, but they do not yet define one deterministic way for an agent or client to compose those capabilities into a repeatable local governance run.

## Findings

### Finding 1: Missing Local Orchestration Layer

Severity: Medium

V5 adds standards orchestration, V6 adds bundles and reports, V7 adds strategy and what-if planning, and V8 adds playbooks and calibration. These are strong pieces, but the roadmap does not yet define a single local orchestration model that says which tools run in which order for common scenarios.

Impact: Agents may still choose the wrong sequence, over-call tools, skip verification, or bury users in fragmented outputs.

Recommendation: V9 should define local orchestration recipes and scenario plans that compose existing tools into one predictable local run.

### Finding 2: Too Much Intelligence Without Enough Output Normalization

Severity: Medium

V5-V8 create more reports, summaries, profiles, playbooks, strategy maps, drills, and calibration outputs. The docs do not yet define a shared result envelope for cross-tool output.

Impact: Clients and agents will have to interpret several slightly different shapes, which increases token use and implementation mistakes.

Recommendation: V9 should define normalized local result shapes for decisions, findings, evidence, next actions, proof requirements, and handoff text.

### Finding 3: Token Budget Is Implied, Not Enforced

Severity: Medium

V5, V6, and V8 mention concise output, token-budgeted summaries, and compact profiles, but they do not define a systematic local context-budget policy.

Impact: The MCP can become helpful but expensive: too much guidance, too many findings, and too much explanation can slow agents down.

Recommendation: V9 should add context-budget governance: compact, standard, and full output modes; truncation rules; evidence budgeting; and summary precedence.

### Finding 4: Evaluation Coverage Grows, But Scenario Acceptance Is Not Yet Named

Severity: Low

V5-V8 include eval suites, drills, calibration, and rule coverage. What is missing is a clear scenario-level acceptance model that says a local run is good enough for a fresh app, a feature, a refactor, a security-sensitive change, or a documentation update.

Impact: Individual tests can pass while the overall workflow remains hard to judge.

Recommendation: V9 should add local scenario acceptance profiles with expected inputs, tool sequence, gate behavior, and pass/fail criteria.

### Finding 5: Productization Boundaries Are Strong

Severity: Positive

Every V5-V8 scope explicitly excludes hosted service implementation, database/storage, GitHub workflows, CLI UX, typed SDKs, billing, accounts, teams, dashboards, and remote policy management.

Impact: The roadmap stays focused on making the MCP concrete before adoption channels are added.

Recommendation: Preserve these exclusions in V9.

### Finding 6: Older V4 Wording Made V5 Sound Like The Productization Gate

Severity: Medium

The V4 scope and V4 audit originally said deferred hosted, database/storage, GitHub, CLI, and SDK work was "blocked until V5" or would "move to V5" after backend architecture was concrete. That conflicts with the later V5-V8 direction, where V5 and beyond deliberately stay local-first.

Impact: Future agents could misread the roadmap and reintroduce productization work into V5-V9 even though the current product strategy is to make the MCP concrete first.

Recommendation: Treat these items as deferred until a later backend/productization phase, not as V5-V9 deliverables.

Status: Corrected in `docs/v4-scope.md` and `docs/v4-scope-audit.md`.

### Finding 7: Source Provenance Needs To Flow Through Later Workflow Layers

Severity: Medium

V3 and V4 strongly define source-backed stack packs from ingested `llms.txt` snapshots, including source identity, evidence, detector mapping, conflict review, examples, and validation. V5-V8 build orchestration, simulation, strategy maps, playbooks, drills, and calibration on top of those packs, but they do not explicitly say that source provenance should remain visible through those later outputs.

Impact: A rule can start source-backed, then become opaque once it appears inside a playbook, scenario report, strategy map, or normalized result.

Recommendation: V9 should preserve source and evidence routing across orchestration recipes, scenario acceptance, result normalization, and final handoff summaries.

### Finding 8: Built-In Skills Catalog Context Should Be Part Of Recipe Selection

Severity: Low

V3 established that architect-mcp ships a built-in advisory skills catalog and can accept external skill metadata as advisory only. V5-V8 mention agent teaching, handoff quality, rule authoring, and documentation intelligence, but do not explicitly connect recipes to the skills catalog.

Impact: The MCP may miss useful local recommendations such as MCP config security review, verification-before-completion, agent instruction quality, or eval discipline when selecting a local workflow.

Recommendation: V9 should let local orchestration recipes include optional advisory skill-pattern recommendations while keeping them non-authoritative and non-executable.

### Finding 9: Artifact Quality Is A Cross-Cutting Gate, Not Only Documentation Intelligence

Severity: Low

V8 includes documentation intelligence, but existing V3 artifact quality work covers `AGENTS.md`, `llms.txt`, setup commands, test commands, build commands, boundaries, verification rules, and security guidance. V5-V8 do not clearly say that those artifact checks should participate in scenario acceptance.

Impact: A local run could pass code/review checks but leave agent-facing docs stale, weak, or missing operational instructions.

Recommendation: V9 should include artifact-quality routing in scenario acceptance profiles and tool-loop gates.

## V9 Recommendation

V9 should be a local operating model layer.

It should not add new product surfaces. It should make the already-planned local intelligence easier for agents and clients to use correctly by defining deterministic scenario orchestration, output normalization, context budgets, local acceptance profiles, source/evidence routing, artifact-quality gates, and explainable run summaries.

## Non-Goals To Preserve

- Hosted service or hosted API implementation.
- Database/storage, including schema, persistence, migrations, review history, or memory storage.
- GitHub repository, branch, PR, issue, app, OAuth, or comment workflow.
- CLI UX.
- Typed client SDK.
- Billing, accounts, teams, dashboards, or remote policy management.
