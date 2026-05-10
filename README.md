# architect-mcp

<p align="center">
  <a href="https://github.com/tonycdr-prog/architect-mcp/releases/tag/v0.1.0"><img alt="release v0.1.0" src="https://img.shields.io/github/v/release/tonycdr-prog/architect-mcp?style=for-the-badge&label=release"></a>
  <a href="https://github.com/tonycdr-prog/architect-mcp/actions/workflows/ci.yml"><img alt="CI status" src="https://img.shields.io/github/actions/workflow/status/tonycdr-prog/architect-mcp/ci.yml?branch=main&style=for-the-badge&label=ci"></a>
  <a href="https://github.com/tonycdr-prog/architect-mcp/blob/main/LICENSE"><img alt="MIT license" src="https://img.shields.io/github/license/tonycdr-prog/architect-mcp?style=for-the-badge"></a>
  <img alt="Node 20+" src="https://img.shields.io/badge/node-%3E%3D20-339933?style=for-the-badge">
</p>

<p align="center">
  <img alt="MCP server" src="https://img.shields.io/badge/MCP-server-111827?style=flat-square">
  <img alt="local first" src="https://img.shields.io/badge/local--first-default-2563eb?style=flat-square">
  <img alt="agent work gate" src="https://img.shields.io/badge/agent%20work%20gate-core-7c3aed?style=flat-square">
  <img alt="hosted safe" src="https://img.shields.io/badge/hosted--safe-stateless-059669?style=flat-square">
  <img alt="verification required" src="https://img.shields.io/badge/verification-required-d97706?style=flat-square">
</p>

architect-mcp keeps coding agents honest. It is a local-first agent work gate that clarifies intent before edits, constrains the plan, reviews implementation drift, and requires verification evidence before completion.

## Launch Snapshot

| Surface | What It Gates |
| --- | --- |
| Core work gate | `grill_me`, pre-edit contracts, plan review, drift review, final/session honesty checks |
| Advanced maturity | Stack packs, standards intelligence, governance evals, operating-model evals |
| Hosted mode | Stateless `/mcp`, explicit file summaries, no local workspace scanning |
| Release gate | `rm -rf dist && npm run release:check` |

## Capabilities

- Pressure-test vague project or implementation requests before code edits.
- Create pre-edit contracts with likely files, non-goals, assumptions, and verification checks.
- Review build plans and proposed file plans before an agent writes files.
- Review changed files or proposed plans against the pre-edit contract.
- Review final responses for changed, verified, assumptions, not-done, and evidence honesty.
- Review complete agent sessions across intent, contract drift, verification, memory, and final response quality.
- Keep hosted mode stateless and prevent local filesystem or file-writing tools from being exposed.
- Keep advanced standards, stack-pack, governance, repo-quality, and productization criteria available for teams that opt in.

## Default Work Gate

The default MCP surface exposes only the core work-gate tools:

- `grill_me`
- `create_pre_edit_contract`
- `review_build_plan`
- `review_proposed_file_plan`
- `review_repo_structure`
- `review_implementation_against_contract`
- `review_agent_final_response`
- `review_agent_session`

Use the advanced surface when you need the full standards, pack-authoring, governance, eval, and productization criteria:

```bash
ARCHITECT_MCP_TOOL_SURFACE=advanced architect-mcp
```

The historical V3-V10 labels remain in tool names, scripts, tests, and document filenames for compatibility. Public documentation frames them as advanced maturity criteria rather than public product versions.

## Advanced Maturity Criteria

The advanced surface keeps the existing local-first criteria without making them part of first-run onboarding:

- Source-backed packs exist for Hono, Zod, Vitest, Auth0, Stripe, Expo, and Vercel AI SDK.
- Executable detectors cover auth boundaries, payment server-only rules, test policy, validation boundaries, route thinness, screen/component size, and AI tool safety.
- Client recipe examples and a lightweight client wrapper live under `examples/`.
- MCP config security can review parsed configs and locally scan repo/user config files when local tools are enabled.
- Final-response review enforces changed, verified, assumptions, remaining work, and evidence honesty.
- Governance evals, operating-model evals, productization boundary evals, artifact quality checks, stack-pack lifecycle tests, and self-contract review are covered in tests.

## Advanced Tool Reference

- `list_stack_packs`: returns available stack-specific architecture packs.
- `validate_stack_packs`: validates pack files against the pack quality bar.
- `list_foundation_packs`: returns versioned foundation packs for harness, testing, repo structure, repo hygiene, and CI gates.
- `validate_foundation_packs`: validates foundation-pack metadata and policy shape.
- `stack_pack_expansion_strategy`: returns the priority stacks and quality contract for stack-pack expansion.
- `discover_llms_sources`: lists known upstream `llms.txt` sources for stack-pack rule generation.
- `fetch_llms_source`: fetches an upstream `llms.txt` and returns a hash-stamped snapshot.
- `ingest_llms_txt`: fetches an upstream `llms.txt` and proposes a stack-pack candidate from it.
- `list_ingested_llms_sources`: lists local ingested snapshots under `stack-sources/ingested/`.
- `derive_stack_pack_from_llms_source`: creates a source-backed candidate from a local ingested snapshot and runs quality/conflict review.
- `propose_stack_pack_rules`: proposes candidate stack-pack rules from local source text or an llms.txt-style snapshot.
- `review_stack_pack_candidate`: reviews candidate stack-pack rules before promotion.
- `promote_stack_pack_candidate`: validates and returns a promotable pack object without writing files.
- `promote_stack_pack_to_files`: validates a reviewed candidate and returns or writes `packs/<id>.json` plus `packs/manifest.json` updates.
- `analyze_stack_pack_conflicts`: reports duplicate or overlapping stack-pack rules before promotion.
- `diff_stack_pack_versions`: compares two stack-pack versions and flags breaking rule changes.
- `list_repo_profiles`: returns reusable profiles like `expo-react-native`, `express-drizzle`, and `mixed-monorepo`.
- `get_repo_profile`: returns one profile by id.
- `infer_repo_layout`: infers a canonical-to-existing repo layout mapping from file paths.
- `interpret_implementation_intent`: interprets vague novice implementation requests and returns a guided-yolo decision.
- `classify_ambiguity_risk`: returns the stoplight, blast radius, change type, and confirmation decision.
- `create_pre_edit_contract`: creates a small implementation contract before risky edits.
- `review_implementation_against_contract`: checks changed files or proposed plans against the pre-edit contract.
- `record_assumption`: returns a stateless assumption-ledger entry for client persistence.
- `load_triggered_stack_guidance`: loads relevant local ingested `llms.txt` guidance when vague terms or stack hints trigger it.
- `extract_harness_memory`: proposes stateless memory entries from harness intent, contracts, session summaries, or user statements.
- `apply_harness_memory`: selects relevant non-red memory proposals within a token budget and discloses what was applied.
- `review_memory_relevance`: reviews memory proposals for relevance, sensitivity, and unsafe silent application.
- `list_skill_catalog`: lists built-in advisory skill patterns and optional client-supplied skill metadata.
- `recommend_skills_for_project`: recommends relevant skill patterns for a request or project without executing skill logic.
- `review_supplied_skills`: reviews external skill metadata before using it as advisory source material.
- `review_mcp_config_security`: audits MCP config objects for hardcoded secrets, shell execution, unpinned packages, and unapproved servers.
- `run_v3_eval_harness`: runs deterministic advanced behavior evals for harness, memory, MCP security, artifact quality, and stack-pack workflows.
- `score_agent_artifacts`: scores generated `AGENTS.md` and `llms.txt` content for agent-operational quality.
- `list_client_integration_recipes`: returns executable client call recipes for pre-edit gates, CI review, memory review, and stack-pack promotion.
- `review_agent_final_response`: checks final agent responses for changed, verified, assumptions, remaining work, and evidence honesty.
- `review_agent_session`: combines intent, contract drift, memory relevance, final response, and verification honesty into one harness report.
- `audit_hosted_tool_policy`: classifies tools as `hosted-safe`, `local-only`, or `future-adapter`.
- `score_stack_packs`: scores stack packs for source quality, detector coverage, examples, tests, and boundaries.
- `stack_pack_coverage_matrix`: reports pack, ingested `llms.txt`, detector, test, and docs coverage by stack.
- `scan_mcp_config_files`: local-only scan of `.mcp.json`, Cursor, Claude Desktop, and Codex MCP config files before security review.
- `resolve_standards_profile`: explains selected minimal, balanced, or strict local standards.
- `explain_review_findings`: turns findings into plain-English meaning, fix shape, next tool, and proof guidance.
- `simulate_policy_gate`: previews review findings under summary, migration, CI, strict, and custom gates.
- `analyze_standards_conflicts`: ranks duplicate or overlapping standards and suggests resolutions.
- `score_repo_profile_fit`: scores built-in repo profiles against a brief and file summaries.
- `review_contract_lifecycle`: reports contract maturity, changelog, deprecations, replacements, and noise risk.
- `run_v5_eval_harness`: runs deterministic standards-intelligence maturity evals.
- `list_policy_bundles`: lists local versioned policy bundles.
- `validate_policy_bundles`: validates local policy-bundle metadata.
- `preview_policy_bundle`: previews a policy bundle against supplied brief and findings.
- `summarize_session_continuity`: builds stateless continuation context from supplied session summaries.
- `cluster_review_findings`: clusters findings by root cause and suggests the first fix.
- `generate_local_report_artifact`: returns Markdown and JSON report content without writing files.
- `analyze_regression_coverage`: maps known agent failure patterns to fixture coverage gaps.
- `review_pattern_card`: validates portable local pattern cards.
- `preview_pattern_card`: previews whether a pattern card applies to a brief.
- `run_v6_eval_harness`: runs deterministic local-governance maturity evals.
- `create_architecture_strategy_map`: maps goals, risks, stack choices, standards, and verification.
- `compare_standards_profiles`: compares minimal, balanced, and strict standards profiles.
- `preview_change_what_if`: forecasts review outcome, blast radius, and verification needs before edits.
- `diagnose_agent_behavior`: detects agent failure patterns from supplied session summaries and reports.
- `draft_rule_candidate`: drafts advisory rule candidates from findings, source text, or examples.
- `compare_review_trends`: compares two supplied reports for new and resolved findings.
- `render_governance_pack`: renders local standards as a human-readable governance pack.
- `run_v7_eval_harness`: runs deterministic strategic-planning maturity evals.
- `review_standards_refactor`: suggests splits, merges, detector families, and example improvements for standards.
- `minimize_policy_set`: recommends keep, defer, or drop choices for local policy rules.
- `select_review_playbook`: selects a local playbook for feature, refactor, security, docs, or dependency work.
- `review_playbook_conformance`: checks supplied tools and verification against a playbook.
- `check_agent_collaboration_plan`: checks file ownership and do-not-touch boundaries.
- `run_failure_mode_drills`: runs local drills for skipped verification, broad rewrites, leaks, and rule overreach.
- `calibrate_rule_impact`: previews severity, confidence, and gate-impact adjustments.
- `review_documentation_intelligence`: detects stale docs, missing tool mentions, and weak agent artifacts.
- `run_v8_eval_harness`: runs deterministic governance-automation maturity evals.
- `select_local_orchestration_recipe`: selects deterministic local operating-model recipes for common agent-work scenarios.
- `evaluate_scenario_acceptance`: checks scenario-level acceptance across intent, contract, review, verification, and artifacts.
- `normalize_mcp_result`: normalizes outputs into status, stoplight, evidence, assumptions, proof, warnings, and handoff.
- `plan_context_budget`: plans compact, standard, or full output budgets.
- `route_evidence`: assigns evidence ids across findings, verification checks, and source provenance.
- `create_local_dry_run_plan`: previews recipe, gates, standards, verification, and final-response contract before edits.
- `review_tool_loop_quality`: detects skipped interpretation, missing contracts, review gaps, and incomplete final responses.
- `run_v9_eval_harness`: runs deterministic operating-model evals across the advanced maturity criteria.
- `get_v10_productization_blueprint`: returns the hosted product API, storage, repository, dashboard, and implementation-slice contract.
- `create_v10_implementation_slice_plan`: returns productization implementation slices with required MCP pre-edit and post-edit gates.
- `plan_primer_dashboard`: returns Primer React dashboard screens, components, data sources, and accessibility checks.
- `validate_v10_productization_boundary`: checks hosted API, storage, dashboard, policy, and billing boundaries for productization regressions.
- `run_v10_eval_harness`: runs deterministic productization boundary evals.
- `build_quality_requirements_profile`: turns interview answers into a quality requirements profile with missing questions and confidence.
- `evaluate_repo_plan_quality`: evaluates proposed stack/repo plans with hard gates, rubrics, follow-up questions, and anti reward-hacking warnings.
- `audit_generated_repo_quality`: audits generated repo quality after generation using the same gates and rubrics.
- `suggest_quality_followup_questions`: returns focused follow-up questions when requirements, plan confidence, or quality gates are weak.
- `run_repo_quality_eval_scenarios`: runs deterministic scenarios for repo quality gates and reward-hacking protections.
- `start_project_intake`: returns the next clarifying question and a recommended answer.
- `grill_project_brief`: stress-tests a project brief and returns readiness, missing fields, and the next question.
- `grill_me`: runs the first-class `/grill-me` loop: pressure-test the brief, select stack packs, and optionally produce contract artifacts.
- `continue_grill_me`: applies one answer to a project brief and reruns `/grill-me`.
- `generate_architecture_contract`: turns a project brief into JSON plus Markdown.
- `generate_agent_instructions`: renders an architecture contract as agent instruction text.
- `generate_agents_md`: renders `AGENTS.md`.
- `generate_cursor_rules`: renders `.cursor/rules/architecture.mdc`.
- `generate_repo_artifacts`: returns `AGENTS.md`, `docs/architecture-contract.md`, and Cursor rules.
- `generate_repo_scaffold_plan`: returns a concrete directory/file creation plan.
- `review_build_plan`: checks ordered build-plan slices before implementation starts, including exact verification commands from the brief or contract.
- `review_proposed_file_plan`: checks a proposed build/scaffold file plan before an agent writes files.
- `review_repo_structure`: reviews supplied file summaries, suitable for hosted MCP usage.
- `review_local_workspace`: scans a local path, suitable for stdio/local usage.
- `create_review_baseline`: creates a baseline from current findings so later reviews can suppress existing debt.
- `validate_architecture_contract`: checks contract metadata, stack pack compatibility, and required guardrail sections.
- `validate_repo_artifacts`: generates and validates repo artifacts before an agent applies them.
- `self_review_architect_mcp`: runs `/grill-me`, contract generation, repo review, and lifecycle classification against the current architect-mcp workspace.
- `diff_architecture_contracts`: compares two contracts and reports breaking architecture-rule changes.
- `mcp_readiness_report`: summarizes pack validation, tool schema policy, self-review, hosted safety, and contract/artifact validation.

## `/grill-me` Loop

`grill_me` is the product loop. It stays local-first: no database, accounts, or hosted state are required.

The tool returns:

- `blockers`: missing answers that should stop implementation.
- `challenges`: pressure-test questions that expose architecture risk.
- `selectedStackPacks`: inferred or requested pack ids.
- `archetype`: inferred app shape, such as `saas-dashboard`, `mobile-field-app`, or `ai-workflow-tool`.
- `foundationPacks`: repo setup, testing, agent harness, and CI gate rules that apply before framework-specific guidance.
- `buildPlan`: ordered implementation slices with files, checks, and stop conditions.
- `updatedBrief`: the brief after an optional one-turn answer is applied.
- `specCompleteness`: a separate score for spec quality across users, workflows, stack, data ownership, verification, harness, risk, and pressure tests.
- `contract` and `markdown`: generated when the brief is ready or `includeContract` is true.
- `artifacts` and `scaffoldPlan`: optional repo outputs for agents to apply deliberately.

The expected flow is:

```text
brief -> /grill-me -> build plan -> contract -> repo artifacts -> review gate -> implementation
```

Implementation should not start while `/grill-me` still reports blocker questions.

The product focus is agent operating discipline: intake readiness, scoped implementation contracts, source-backed standards, architecture/repo governance, verification honesty, and optional memory continuity. External repos are useful smoke tests, but they should not drive product behavior unless they expose a general agent-workflow failure mode.

Some architecture challenges are blockers even when the basic brief fields are filled:

- Frontend plus database with no named backend/server boundary.
- Auth without a server-side trust boundary for sessions, roles, and tenant checks.
- Hosted tools that mention filesystem/local-path scanning without a safety boundary.
- Blocking/CI enforcement without exact verification commands.

`/grill-me` also pressure-tests common failure areas before implementation:

- frontend feature/screen/state ownership
- backend route/service/adapter boundaries
- database schema/migration/repository discipline
- repo-owned agent harness files such as `AGENTS.md`, architecture contracts, and editor rules
- verification commands and evidence expectations
- security-sensitive boundaries such as auth, payments, secrets, and hosted filesystem access

Before writing files, clients can send a proposed scaffold to `review_proposed_file_plan`. It flags plans that put many workflows into `App.tsx`, `server.ts`, `index.ts`, or similar entry files, UI plans without feature/component/screen boundaries, database plans without server-owned data access, and plans missing `AGENTS.md` or `docs/architecture-contract.md`.

Clients can also send the generated `buildPlan` to `review_build_plan`. Build-plan slices are contracts: each slice must declare inputs, outputs, allowed directories, forbidden files, checks, and a stop condition. This lets `/grill-me` enforce build order before code is written.

`review_build_plan` can receive `allowedChecks` or an architecture `contract`. When provided, slice checks must match the brief/contract verification commands instead of inventing new proof steps.

`grill_me` supports a lightweight multi-turn loop by accepting an optional `answer` object:

```json
{
  "brief": { "idea": "A small admin app" },
  "answer": {
    "field": "users",
    "value": "Ops admins who need to triage customer records."
  }
}
```

The response includes `updatedBrief`, so a client can apply one answer, ask the next question, and rerun until blockers are gone.

`continue_grill_me` is a first-class alias for that multi-turn path when a client wants a dedicated continuation tool instead of overloading `grill_me`.

## Guided YOLO Harness

The harness protects novice-friendly prompts without killing flow. The default mode is `guided-yolo`: concrete low-risk requests proceed, vague low-risk requests proceed with logged assumptions, and vague risky requests ask one plain-English confirmation question before edits.

Use `interpret_implementation_intent` before implementation when the user says things like "fix this with best practices", "make auth better", "clean this up", "secure it", "optimize database", or "refactor properly". The tool returns:

- `decision`: `proceed`, `proceed_with_assumptions`, `confirm_before_edit`, or `block_until_clarified`.
- `stoplight`: `green`, `yellow`, or `red`.
- `blastRadius`, `changeType`, assumptions, non-goals, verification, evidence, and a handoff summary.
- `confirmationPrompt` when the agent should ask "I think you mean X; I plan to do Y; is that right?"

When vague terms or stack hints trigger best-practice guidance, `load_triggered_stack_guidance` prefers local ingested `llms.txt` snapshots and uses derived stack-pack evidence. Missing or stale snapshots produce warnings but do not block the harness.

The intended flow is:

```text
user request -> interpret_implementation_intent -> load_triggered_stack_guidance -> create_pre_edit_contract -> implementation -> review_implementation_against_contract -> honest final output
```

## Stateless Memory Proposals

The memory layer is an abstraction, not storage. The current implementation does not create repos, write files, call GitHub, or require a database. It returns structured memory proposals that a future adapter can persist through a user-owned GitHub repo, branch, and PR workflow.

Memory is risk-tiered:

- `green`: low-risk preferences or repo facts, eligible for auto-store by a client policy.
- `yellow`: architecture decisions, accepted tradeoffs, or assumptions, suitable for batched review.
- `red`: sensitive, destructive, security-heavy, or secret-like memory, requiring confirmation or discard.

The intended flow is:

```text
harness output -> extract_harness_memory -> review_memory_relevance -> apply_harness_memory -> future GitHub/storage adapter
```

`apply_harness_memory` loads only relevant proposals within a token budget and returns disclosures such as "Using remembered project decision..." so memory never silently overrides the current user request.

## Advanced Source Material

The advanced source-material notes live in `docs/v3-source-material.md`. They capture useful local skill patterns and product directions without adding runtime dependencies:

- deeper source-backed stack packs from local `llms.txt` snapshots
- client harness recipes for pre-edit and post-edit gates through `list_client_integration_recipes`
- eval-harness fixture classes through `run_v3_eval_harness`
- local skill/catalog ingestion
- MCP config security review through `review_mcp_config_security`
- agent instruction quality checks through `score_agent_artifacts`
- memory scope compatibility
- evidence-before-completion verification policy

## Skills Catalog

The skills catalog is built in. It does not assume users have local Codex skills installed. The catalog stores reusable patterns architect-mcp understands, such as MCP tool discovery, MCP config security, scoped memory, agent instruction quality, verification honesty, and eval fixture discipline.

Clients can optionally pass external skill metadata to `recommend_skills_for_project`, but supplied skills are advisory only. `review_supplied_skills` checks that external metadata has recommendation patterns and does not ask agents to execute arbitrary logic or ignore the current user request.

Executable client call examples live under `examples/`, including guided-yolo pre-edit gating, MCP config security review, and stack-pack promotion.

The current advanced coverage and remaining future work are tracked in `docs/v3-status.md`.
See `docs/use-on-a-repo.md` for local and hosted-safe workflows. Hosted API boundaries are sketched in `docs/hosted-api-shape.md`; this is a contract target, not a hosting implementation.
Compatibility references remain available in the historical files: `docs/v1-v3-completion-audit.md`, `docs/v4-scope-audit.md`, `docs/v4-scope.md`, `docs/v5-v8-scope-audit.md`, `docs/v5-scope.md`, `docs/v6-scope.md`, `docs/v7-scope.md`, `docs/v8-scope.md`, `docs/v9-scope.md`, and `docs/v10-productization-implementation.md`.

## Pack Authoring

Stack packs live in `packs/*.json`. A useful pack should be concrete enough for both generation and review.

Required rule fields:

- `version`: semver version for pack behavior.
- `rationale`: why this pack exists and what failure mode it prevents.
- `sources`: source or rationale notes that make the pack auditable.
- `name`: short rule label.
- `rule`: durable architecture rule.
- `severity`: `error` or `warning`.
- `trigger`: what observable condition should make the rule relevant.
- `recommendation`: what the agent should do instead.
- `appliesToPaths`: path globs where the rule matters.
- `goodExample` / `badExample`: small examples when useful.

Run the validator through MCP with `validate_stack_packs`, or locally through tests.

## Repo Layout Mapping

Stack packs use canonical paths such as `src/features`, `src/server`, and `src/db`. Existing repos can keep their own structure by passing `brief.repoLayout.pathMap`.

Example:

```json
{
  "repoLayout": {
    "pathMap": {
      "src/features": ["client", "admin/react/pages"],
      "src/shared": ["shared"],
      "src/shared/ui": ["client/components", "admin/react/components"],
      "src/server": ["server"],
      "src/server/routes": ["server/routes"],
      "src/server/services": ["server/services", "server/lib"],
      "src/db": ["supabase"],
      "src/db/schema": ["shared"],
      "src/db/migrations": ["server/migrations"],
      "tests": ["client", "server", "admin"]
    }
  }
}
```

Use `infer_repo_layout` to generate a starter mapping from a file list, then adjust it for the repo’s actual architecture.

Current executable trigger families:

- thin route/controller files
- transport-specific logic in MCP tool handlers
- oversized React god components
- broad client component boundaries
- UI database access
- direct environment reads
- schema changes requiring migration discipline
- Supabase server/client boundary leaks
- hosted filesystem scanning exposure
- import graph boundaries, including transitive UI-to-database paths, shared-to-feature paths, route files without service/use-case edges, and circular dependencies

## Stack Pack Workflow

Stack-pack expansion can use upstream `llms.txt` sources, but promotion is still review-gated. Known upstream sources are cataloged in `stack-sources/llms-sources.json` and exposed by `discover_llms_sources`.

Use `fetch_llms_source` to create a source snapshot with URL, fetch timestamp, content type, byte count, and SHA-256 hash. Use `ingest_llms_txt` to fetch and propose candidate rules in one step.

The current known-source catalog has been ingested under `stack-sources/ingested/`; `index.json` records URL, fetch time, hash, byte count, local path, and status for each upstream source. Failed fetches remain in the index with error details so stale or blocked `llms.txt` URLs are visible.

Use `derive_stack_pack_from_llms_source` when a snapshot is already ingested. It returns matched evidence headings, source snapshot provenance, candidate rules, quality violations, and overlap warnings against existing packs. You can also put reviewed source snapshots under `stack-sources/` or pass source text directly to `propose_stack_pack_rules`. The tool returns candidate rules with source metadata, rationale, triggers, detector metadata, path scopes, and examples.

Candidate flow:

```text
discover_llms_sources -> fetch_llms_source -> ingest_llms_txt -> derive_stack_pack_from_llms_source -> review_stack_pack_candidate -> analyze_stack_pack_conflicts -> promote_stack_pack_candidate -> write pack JSON -> update manifest -> validate_stack_packs
```

Packs support structured detector metadata:

- `triggerKind`: executable detector family, such as `line-threshold`, `import-boundary`, `direct-db-access`, `client-boundary`, or `route-thinness`.
- `detectors`: short descriptions of how the rule maps to reviewer behavior.

The current priority stack order is documented in `docs/stack-pack-strategy.md`: Next.js, React/Vite, Node API, Postgres, Supabase, then Expo React Native.

The repo also ships `llms.txt` so agents and tooling can discover architect-mcp flows, tools, finding codes, and pack-authoring rules.

## Star History

<a href="https://www.star-history.com/#tonycdr-prog/architect-mcp&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tonycdr-prog/architect-mcp&type=Date&theme=dark">
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tonycdr-prog/architect-mcp&type=Date">
    <img alt="Star History Chart for tonycdr-prog/architect-mcp" src="https://api.star-history.com/svg?repos=tonycdr-prog/architect-mcp&type=Date">
  </picture>
</a>

## Run Locally

The npm registry name `architect-mcp` is owned by a different package. This project keeps the `architect-mcp` command name, but package distribution uses the scoped package identity `@tonycdr-prog/architect-mcp` and GitHub release tarballs until npm publishing is configured for that scope.

Install the launch package from the GitHub release tarball:

```bash
npm install -g https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.0/tonycdr-prog-architect-mcp-0.1.0.tgz
architect-mcp
```

For a source checkout:

```bash
npm install
npm run build
node dist/index.js
```

For local MCP clients, point the command at this repo:

```json
{
  "mcpServers": {
    "architect-mcp": {
      "command": "npx",
      "args": [
        "-y",
        "--package",
        "https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.0/tonycdr-prog-architect-mcp-0.1.0.tgz",
        "architect-mcp"
      ]
    }
  }
}
```

For a local checkout, build first and use your own absolute path to `dist/index.js`.

## Run Hosted HTTP Mode

Hosted HTTP mode is stateless and does not need a database. Each `/mcp` request creates a fresh MCP server with no session storage.

```bash
npm install
npm run build
PORT=3000 HOST=0.0.0.0 npm run start:http
```

Endpoints:

- `POST /mcp`: stateless Streamable HTTP MCP endpoint.
- `GET /health`: simple health check for deploy platforms.

Hosted mode intentionally disables `review_local_workspace`, because a public server should not scan arbitrary server paths. Use `review_repo_structure` instead by sending file summaries from the client/agent.

For existing repos, include both source file summaries and directory paths. Directory paths let the reviewer validate mapped folders that may not contain reviewed source files.

Review modes:

- `strict`: return every finding.
- `summary`: return grouped findings plus priority findings.
- `ci`: return only blocker/error findings.
- `audit`: review an established repo without requiring generated agent harness artifacts.
- `migration`: suppress lower-value line-count noise for mature repos.

Use `.architectignore` or `ignorePatterns` to suppress project-specific generated datasets, fixtures, mocks, and snapshots. The built-in review noise filter already excludes common lockfiles, public/static assets, archive folders, migration metadata, media/font files, and generated outputs.

Findings include:

- `code`: stable identifier such as `ARCH001_OVERSIZED_FILE` or `ARCH009_SERVER_ENV_IN_UI`.
- `confidence`: `high`, `medium`, or `low`, based on how directly observable the rule violation is.
- `severity`: `error` or `warning`.

Line-count warnings are category-aware. UI, route, service, schema, declaration, test, script, config, changelog, and docs/data files each use their own threshold so generated or naturally broad files do not drown out real architecture issues.

Baselines suppress known findings by `code` plus optional `path` and `message`. A repo can create a baseline once, pass it back into `review_repo_structure` or `review_local_workspace`, and use CI mode to catch new blocker findings without forcing immediate cleanup of every historical warning.

The domain lifecycle classifier separates current review findings into:

- `newFindings`: current findings not present in the baseline.
- `baselineFindings`: current findings suppressed as known debt.
- `acceptedFindings`: current findings explicitly accepted with a reason.
- `resolvedFindings`: baseline entries no longer present in the current review.

When a review tool receives a `baseline`, its response includes this `lifecycle` breakdown alongside the filtered report.

Review reports also include a `gate`:

- `pass`: no configured threshold was exceeded.
- `warn`: warnings or score missed the configured target.
- `fail`: error findings exceeded the configured error threshold.

By default, CI mode fails on any error and warns on any warning. Callers can override `gate.maxErrors`, `gate.maxWarnings`, and `gate.minScore` for migration periods.

Generated contracts include contract metadata:

- `contractVersion`: version of the generated contract shape.
- `generatedBy.tool`: always `architect-mcp`.
- `generatedBy.version`: package/tool version that generated the contract.
- `generatedBy.generatedAt`: ISO timestamp for auditability.

Contract inputs accepted by review/validation tools are schema-checked. Unknown fields are rejected on strict tool inputs such as briefs, file summaries, baselines, gates, and architecture contracts.

Generated artifacts are validated before being returned. Required artifact markers include selected stack packs, review gate guidance, baseline lifecycle guidance, and architecture review instructions.

Generated contracts include pre-coding checklist, app-generation guardrails, verification policy, and agent harness setup sections so agents have concrete instructions before creating files.

Generated repo artifacts now include `AGENTS.md`, `docs/architecture-contract.md`, `.cursor/rules/architecture.mdc`, `.architectignore`, and `docs/build-plan.md`.

Foundation packs live in `foundation-packs/*.json` so repo setup policy can be versioned and validated like stack packs. `foundation-packs/manifest.json` tracks version and SHA-256 hashes.

Repo review can accept a `buildPlan`. When present, implementation drift checks compare files against the plan’s allowed directories, forbidden files, and planned outputs. This catches generated app attempts that skip harness setup, concentrate responsibilities in entry files, write files outside the plan, bypass stack boundaries, or omit tests for feature slices.

Critical tools expose MCP `outputSchema` and return `structuredContent` as well as text JSON. Tool-handled runtime failures use a predictable error shape:

```json
{
  "error": {
    "code": "ARCHITECT_TOOL_ERROR",
    "message": "Human-readable failure",
    "details": null
  }
}
```

Pack changes are tracked in `packs/manifest.json`. If pack content changes, update the manifest and bump the pack version when rule behavior changes.

Baseline safety:

- Code-only baseline entries are allowed only for lower-risk warning findings.
- Error or high-confidence findings require `path` or `message` before they can be suppressed.
- Accepted baseline findings require a human-readable `reason`.

Hosted safety:

- Hosted mode must not expose `review_local_workspace`, `scan_mcp_config_files`, or `promote_stack_pack_to_files`.
- Attempts to call local workspace scanning when it is not registered return an MCP tool error.

MCP readiness should pass before release: `npm run release:check` runs the full advanced readiness gate and bootstraps the compiled readiness entrypoints from a clean checkout. `npm run check:v3` runs typecheck, tests, build, audit, package dry-run inclusion, and `mcp_readiness_report`. The historical compatibility scripts `npm run check:v5`, `npm run check:v6`, `npm run check:v7`, `npm run check:v8`, `npm run check:v9`, and `npm run check:v10` still chain the base release check and then run standards-intelligence, governance, operating-model, and productization boundary evals. The underlying readiness tool checks pack validation, policy bundle validation, contract/artifact validation, self-review, hosted safety, advanced staged evals, tool schema policy, and release scripts.

For richer hosted reviews, clients should include optional file-summary signals:

```json
{
  "path": "src/features/billing/BillingClient.tsx",
  "lines": 180,
  "imports": ["@/server/billing"],
  "hasUseClient": true,
  "envAccesses": [],
  "hasDirectDbAccess": false
}
```

Local workspace scans derive those signals automatically. For JavaScript and TypeScript files, the scanner uses the TypeScript parser to extract imports and literal `process.env.NAME` accesses instead of relying only on regular expressions.

Optional environment variables:

- `PORT`: HTTP port. Defaults to `3000`.
- `HOST`: bind host. Defaults to `0.0.0.0`.
- `ALLOWED_HOSTS`: comma-separated host allow-list for DNS rebinding protection.
- `JSON_BODY_LIMIT`: maximum JSON request size for large repo reviews. Defaults to `10mb`.

Railway defaults are included in `railway.json`:

```bash
npm ci && npm run build
npm run start:http
```

When publishing as an npm package, `package.json` includes `files` entries for `dist/`, packs, policy bundles, source metadata, docs, `README.md`, `LICENSE`, and `SECURITY.md` so runtime pack loading and provenance work outside the source checkout. Repo-only TypeScript readiness scripts are intentionally not included in the package tarball, and publish-time manifest preparation strips repo-only npm scripts from the packed `package.json`.

Security reports should follow `SECURITY.md`. Do not put vulnerabilities, secrets, exploit details, or private repository data in public issues.

## Example Brief

```json
{
  "brief": {
    "idea": "A small SaaS dashboard for teams to review AI-generated pull requests.",
    "users": "Engineering leads and solo founders",
    "coreFlows": ["connect repo", "review architecture findings", "export agent instructions"],
    "stack": {
      "frontend": "Next.js",
      "backend": "Node API",
      "database": "Postgres",
      "auth": "Supabase Auth"
    }
  },
  "stackPackIds": ["nextjs", "node-api", "postgres", "supabase"]
}
```

## Design Principle

The server should stay small and composable. Every feature should strengthen the broader agent standards loop:

```text
intent -> standards -> contract -> implementation -> review -> evidence
```

Stack packs, harness checks, memory proposals, hosted adapters, and future GitHub workflows should remain optional layers around that loop. Tests should use synthetic fixtures under `tests/fixtures/`, not external application repos. External repos are useful for manual smoke testing, but the product loop should be validated with owned fixtures that cover clean, messy, existing-layout, migration-baseline, harness, and memory-safety scenarios.
