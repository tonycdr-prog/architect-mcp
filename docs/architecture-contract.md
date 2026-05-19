# Anti-Monolith Architecture Contract

## Stack Packs
- MCP Server: domain logic belongs in src/domain; MCP tool handlers remain transport-neutral wrappers.

## Foundation Packs
- Agent Harness: AGENTS.md, docs/architecture-contract.md, docs/build-plan.md, and editor rules must exist before implementation expands.
- Testing: each rule or generated-plan behavior needs focused tests.
- Repo Structure: organize by responsibility, not by catch-all files.
- CI Gates: new high-confidence architecture errors should fail review.

## App Archetype
- ai-workflow-tool

## Pre-Coding Checklist
- /grill-me has no blocker questions.
- The contract and build plan exist in docs.
- Verification commands are known before implementation starts.

## App Generation Guardrails
- Keep src/domain pure and deterministic.
- Keep src/tools thin and transport-neutral.
- Keep src/infrastructure responsible for filesystem/runtime adapters.
- Add new review behavior with tests before relying on it in gates.

## Agent Harness Setup
- AGENTS.md gives future agents repo instructions.
- docs/architecture-contract.md is the human-readable contract.
- .cursor/rules/architecture.mdc mirrors the same guardrails for editor agents.

## Review Gate
- Default CI gate: zero error findings.
- Baselines must not suppress new high-confidence errors without path/message specificity.
- Architecture review should run after scaffold and major generated changes.
- Launch-stack merge plans are read-only checklists; they must never merge PRs, close issues, tag releases, publish packages, or replace real terminal QA evidence.
- Terminal evidence must record provenance; hosted CI, container, unknown, or missing provenance cannot satisfy final manual Linux/Windows terminal QA.
- Repo-foundry public summaries must omit local workspace paths, staged repo paths, private proof-repo names/URLs, raw command strings, command transcripts, stdout/stderr tails, raw MCP payloads, and token-shaped values.
- Repo-foundry retention decisions must be explicit public-safe evidence only; they must not imply repository deletion occurred unless a separate approved cleanup action actually performed it.

## Baseline Lifecycle
- New findings must be fixed or deliberately accepted with a reason.
- Baseline findings are known debt and should not hide new findings.
- Resolved findings should be removed from baselines.

## Architecture review
- Required commands: npm run typecheck, npm test, npm run build.
