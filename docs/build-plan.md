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
