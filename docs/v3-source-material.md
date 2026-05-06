# V3 Source Material

This document captures source material for V3 without making any of it a runtime dependency. The product rule is still local-first: new integrations should be optional adapters or source-backed rules, not mandatory services.

## Source-Backed Stack Packs

Use ingested `llms.txt` snapshots as the default source for stack-pack depth. A pack rule should be promoted only when it has:

- source identity: source id, URL, snapshot path, fetched timestamp, and SHA-256 hash
- evidence: headings or short excerpts that justify the rule
- detector mapping: executable detector family when possible
- conflict review: duplicate, overlapping, or competing rules checked before promotion
- examples: small good/bad examples for agent-facing clarity

The preferred flow is:

```text
discover_llms_sources -> fetch_llms_source -> ingest_llms_txt -> derive_stack_pack_from_llms_source -> review_stack_pack_candidate -> analyze_stack_pack_conflicts -> promote_stack_pack_candidate -> promote_stack_pack_to_files -> validate_stack_packs
```

Live fetching should remain explicit. Intent interpretation and harness checks should load local ingested snapshots first.

## Harness Integration Recipes

Clients should wire architect-mcp as a pre-edit and post-edit gate:

1. Run `interpret_implementation_intent` for vague or best-practice language.
2. Run `load_triggered_stack_guidance` when stack terms or escalation words appear.
3. Create `create_pre_edit_contract` before risky edits.
4. Apply edits inside the contract scope.
5. Run `review_implementation_against_contract` with changed files and verification status.
6. Final responses should include changed, verified, assumptions, and not done.

Client recipes are exposed through `list_client_integration_recipes` and mirrored as executable JSON examples under `examples/`.

## Eval Harness

The V3 eval harness should test the MCP behavior, not the model. Useful fixture classes:

- novice vague prompts: "fix this with best practices", "secure it", "make auth better"
- implementation drift: broad entry files, route/service boundary leaks, UI-to-DB leaks, skipped harness setup, missing evidence, and widened scope
- stack-pack conflicts: duplicate rules, competing boundaries, weak source evidence
- memory safety: secret-like memory discarded, red memory not silently applied, token budget respected
- hosted/local policy: hosted mode does not expose local filesystem scanning

The useful source pattern is eval discipline: define criteria, create datasets, run scored evaluations, report pass/fail results, and iterate on failures. `run_v3_eval_harness` now runs deterministic V3 behavior evals for harness, memory, MCP security, artifact quality, and stack-pack workflow coverage.

## Skill And Catalog Ingestion

architect-mcp ships a built-in skills catalog for source patterns it understands. Users should not be expected to have any particular local skills installed. A client can optionally supply additional skill metadata, but V3 should not execute arbitrary skill logic. A safe ingestion flow would:

- start with the built-in catalog
- read supplied or discoverable `SKILL.md` metadata and short descriptions only
- classify skills by project need: MCP, security, memory, GitHub, eval, docs, frontend, database
- recommend relevant skills with reason and confidence
- warn when a skill suggests actions outside the current contract
- never treat a skill as authoritative over the current user request or repo contract

The source patterns captured for V3 are:

- MCP tool discovery and CLI call recipes
- MCP config security checks
- scoped, domain-organized memory conventions
- agent instruction structure
- LLM navigation file structure
- evidence before completion claims
- eval setup patterns and pass/fail discipline

## MCP Config Security Review

`review_mcp_config_security` audits MCP config objects for patterns inspired by MCP security-audit:

- hardcoded secrets in args or env
- command substitution, shell chaining, `eval`, `bash -c`, and `curl | sh`
- unpinned packages such as `@latest`
- `npx` without non-interactive behavior in CI contexts
- unapproved or unknown MCP servers
- environment variables used safely instead of inline credentials

This is usable against `.mcp.json`, editor MCP configs, and generated agent setup snippets when clients pass the parsed config object.

## Agent Instruction Quality

Generated `AGENTS.md`, Cursor rules, and `llms.txt` should stay concise and operational. `score_agent_artifacts` checks:

- setup commands, test commands, and build commands are present
- repo boundaries, stack standards, verification rules, and responsibility-splitting guidance are explicit
- generated instructions distinguish human README content from agent-only workflow instructions
- security and secrets guidance is included when tools or hosted mode are involved
- links in `llms.txt` point to useful source files, docs, examples, and contracts

## Memory Scope Compatibility

Current memory scopes map cleanly to common agent memory conventions:

- `user`: long-lived preferences that apply across repos
- `project`: durable decisions and stack defaults for one project
- `session`: short-lived assumptions, confirmations, and handoff notes
- `codebase`: observed repo facts, architecture summaries, and anti-patterns

Memory must disclose when it is applied, stay within token budgets, and never override the current user instruction. GitHub-backed memory should store proposals through branch/PR review rather than silently writing durable state.

## Verification Policy Pack

"Evidence before completion" should be a first-class foundation rule:

- no success claim without a fresh relevant command or inspection result
- final output must state checks run, failed, skipped, or not run
- root-cause claims require evidence
- regression claims require a reproduction or focused test
- broad "done" claims are invalid when contract verification is missing

This policy belongs in generated agent instructions, build-plan review, implementation contracts, and final-output checks.
