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
