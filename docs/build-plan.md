# Build Plan

## App Archetype
- ai-workflow-tool

## Slices
### 1. Agent Harness And Contract
- Inputs: project brief, /grill-me result, selected stack packs
- Outputs: AGENTS.md, docs/architecture-contract.md, .cursor/rules/architecture.mdc
- Allowed directories: docs, .cursor/rules
- Forbidden files: src/App.tsx, src/app/page.tsx, src/server.ts, src/index.ts
- Checks: npm run typecheck, npm test
- Stop after: stop if /grill-me still has blocker questions.

### 2. Responsibility-Based Scaffold
- Inputs: architecture contract, repo layout mapping
- Outputs: responsibility-owned domain, tools, server, infrastructure, and test boundaries
- Allowed directories: src/domain, src/tools, src/server, src/infrastructure, tests, docs
- Forbidden files: src/server.ts, src/index.ts
- Checks: npm test
- Stop after: stop if scaffold review finds missing boundaries.

### 3. Verification And Review Gate
- Inputs: implemented slice, verification commands
- Outputs: test results and architecture review report
- Allowed directories: tests, docs
- Forbidden files: baseline entries without path or reason
- Checks: npm run typecheck, npm test, npm run build
- Stop after: stop if review gate fails.

### 4. Launch Checklist Evidence
- Inputs: discovered PR stack, PR review decisions, explicit required check names, blocker issues, terminal QA issue state, terminal evidence provenance
- Outputs: read-only launch-stack merge plan, public-safe evidence summary with missing required-check evidence, review-aware/provenance-aware launch gate, required-check absence detection, goal ledger update
- Allowed directories: crates/architect-tui/src, crates/architect-tui/tests, docs, tests
- Forbidden files: GitHub mutation scripts, auto-merge flows, release tagging, package publishing
- Checks: cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace, npm run release:check
- Stop after: stop if requested PR changes are unresolved, required review is incomplete, any explicit required check is missing, #136 terminal evidence is missing, #136 evidence lacks manual local/VM provenance, or there is no explicit maintainer waiver.

### 5. Repo-Foundry Public Evidence
- Inputs: repo-foundry smoke report, private repo verification, draft PR verification, command outcomes, retention decision
- Outputs: public-safe foundry smoke summary, retention decision evidence, docs update, goal ledger update
- Allowed directories: crates/architect-tui/src, crates/architect-tui/tests, docs, tests
- Forbidden files: auto-merge flows, GitHub mutation without `--execute --confirm-private-repo-mutation`, automatic repo deletion, public logs containing private proof-repo URLs
- Checks: cargo fmt --check, cargo test -p architect-tui foundry_smoke_public_summary, cargo test --workspace --test foundry_smoke, npm run release:check
- Stop after: stop if the public summary exposes workspace paths, private repo targets, raw commands, command transcripts, stdout/stderr tails, or claims a proof repo was deleted automatically.

### 6. Published Package Smoke Evidence
- Inputs: issue #136 automatable smoke scope, published npm package, hosted Ubuntu and Windows runners
- Outputs: pinned read-only published-package workflow, public docs boundary, goal ledger update
- Allowed directories: .github/workflows, tests, docs
- Forbidden files: interactive terminal automation, adapter execution without `--execute`, auto-closing issue #136
- Checks: node --import tsx --test tests/supplyChain.test.ts, node --import tsx --test tests/publishedPackageSmokeWorkflow.test.ts, npm run docs:build, npm run release:check
- Stop after: stop if hosted CI evidence is described as satisfying manual Linux/Windows terminal QA or if the gate-only JSONL smoke can start an adapter.

### 7. Linux ARM64 TUI Release Assets
- Inputs: issue #241, npm shim Linux ARM64 asset naming, GitHub hosted ARM64 runner support, existing TUI release workflow
- Outputs: Linux ARM64 TUI release matrix entry, Linux ARM64 source-built install-smoke coverage, public docs boundary, goal ledger update
- Allowed directories: .github/workflows, tests, docs
- Forbidden files: published-package ARM64 smoke before a matching release asset exists, release tags, package publishing, auto-closing issue #136
- Checks: node --import tsx --test tests/supplyChain.test.ts, node --import tsx --test tests/tuiShim.test.ts, npm run docs:build, npm run release:check, git diff --check
- Stop after: stop if Linux ARM64 hosted evidence is described as satisfying manual Linux/Windows terminal QA or if existing Linux x64, macOS, or Windows release coverage is removed.

### 8. Prompt Injection And Gate Bypass Threat Model
- Inputs: issue #242, existing work-gate docs, TUI approval/promotion behavior, security reporting policy
- Outputs: public-safe threat-model doc, advisory-versus-enforced gate classification, reproducible bypass fixtures, follow-up issue links, goal ledger update
- Allowed directories: src/domain, tests, docs
- Forbidden files: exploit payloads, secrets, private repository data, cloud moderation dependencies, changes that claim MCP tools sandbox direct shell or file mutation
- Checks: node --import tsx --test tests/workGateThreatModel.test.ts, node --import tsx --test tests/supplyChain.test.ts, npm run docs:build, npm run release:check, git diff --check
- Stop after: stop if public docs overclaim that report-only MCP tools prevent prompt injection, direct file edits, selective tool calls, or fabricated verification evidence without TUI, host, CI, or human enforcement.

### 9. TUI Untrusted Input Labels
- Inputs: issue #244, prompt-injection threat model, TUI session/transcript/review prompt flow
- Outputs: metadata-only untrusted-input labels in TUI sessions, transcript/inspector display, adapter prompt notice, final/session review request labels, docs update, goal ledger update
- Allowed directories: crates/architect-tui/src, crates/architect-tui/tests, src/domain, src/tools/schemas, tests, docs
- Forbidden files: raw exploit payloads, secrets, private repository data, cloud moderation dependencies, claims that labels prevent prompt injection or sandbox models
- Checks: cargo test -p architect-tui untrusted, cargo test --workspace --test workflows, node --import tsx --test tests/finalResponseReview.test.ts tests/agentSessionReview.test.ts, npm run docs:build, npm run release:check, git diff --check
- Stop after: stop if labels include raw untrusted text, if review prompts omit labels for adapter/MCP output, or if docs describe labeling as model-level prevention.

### 10. Non-TUI Work-Gate Completeness Audit
- Inputs: issue #245, prompt-injection threat model, core work-gate sequence, direct-client PR/launch evidence needs
- Outputs: read-only `audit_work_gate_completeness` MCP tool, domain audit for no/partial/stale/out-of-order/complete evidence, schema validation, docs update, goal ledger update
- Allowed directories: src/domain, src/tools, tests, docs
- Forbidden files: TUI mutation paths, filesystem sandbox claims, raw issue/PR/log payload storage, command-success inference without supplied evidence
- Checks: node --import tsx --test tests/workGateCompleteness.test.ts tests/toolResponses.test.ts tests/schemaValidation.test.ts, npm run typecheck, npm test, npm run build, npm run release:check, git diff --check
- Stop after: stop if the audit mutates files, accepts unknown gate names at the tool boundary, reflects raw evidence payloads, or claims it can force direct clients to call every gate.
