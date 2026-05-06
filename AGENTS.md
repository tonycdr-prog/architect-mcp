# AGENTS.md

This repository uses architect-mcp as a local-first standards, architecture, and verification harness for coding agents. Follow these rules before making implementation changes.

## Project Overview
- TypeScript MCP server that helps agents ask better intake questions, generate architecture contracts, load stack standards, review implementation quality, and gate vague or risky work.
- Domain logic lives in `src/domain`; MCP tool wrappers live in `src/tools`; tests live in `tests`.

## Setup Commands
- Install dependencies: `npm install`
- Typecheck: `npm run typecheck`
- Test: `npm test`
- Build: `npm run build`

## Build Order For Coding Agents
1. Run /grill-me and stop if any blocker remains.
2. Generate or update AGENTS.md, docs/architecture-contract.md, docs/build-plan.md, and editor rules.
3. Create scaffold directories by responsibility before feature code.
4. Build one workflow slice at a time.
5. Run verification and architecture review before starting the next slice.

## Testing Instructions
- Run `npm run typecheck`, `npm test`, and `npm run build` after architecture-rule, schema, or tool-surface changes.
- Add or update focused tests for every new MCP tool, review rule, stack-pack behavior, harness behavior, and artifact policy.
- Keep public tool documentation in sync with registered tools; the test suite checks `README.md` and `llms.txt`.

## Code Boundaries
- Keep reusable policy and review behavior in `src/domain`.
- Keep MCP registration, input schemas, and response shaping in `src/tools`.
- Keep stack-pack JSON under `packs/` and foundation pack JSON under `foundation-packs/`.
- Keep generated, vendored, and binary review noise outside source fixtures.

## Do Not Create
- Giant App.tsx, page.tsx, server.ts, route.ts, or index.ts files containing multiple workflows.
- Tool handlers that own domain logic instead of calling domain modules.
- Shared folders without at least two real consumers.
- New architecture rules without tests and finding metadata.

## Architecture Review
- Run npm run typecheck, npm test, and npm run build after architecture-rule changes.
- Run self-review before release-sensitive changes.
- Keep docs/architecture-contract.md and docs/build-plan.md aligned with product behavior.

## Proof Of Completion
- State which checks were run and whether they passed, failed, were skipped, or were not run.
- Do not claim a root cause without evidence from code, test output, or tool output.
- Call out assumptions, remaining gaps, and any intentionally deferred work.
