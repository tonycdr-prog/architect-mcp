# V3 Status

V3 is concrete enough to stay local-first and continue without hosting or CLI work.

## Covered

- MCP config security review through `review_mcp_config_security`.
- Dedicated deterministic V3 eval reporting through `run_v3_eval_harness`.
- Stronger generated `AGENTS.md` and `llms.txt` scoring through `score_agent_artifacts`.
- Source-backed stack-pack promotion through `promote_stack_pack_to_files`, including dry-run outputs, readiness metadata, manifest diffs, overwrite blocking, and explicit version bumps.
- Promoted source-backed stack packs for Hono, Zod, Vitest, Auth0, Stripe, Expo, and Vercel AI SDK.
- Executable detector coverage for auth boundaries, payment boundaries, test policy, validation boundaries, route thinness, screen/component size, and AI tool safety.
- Client integration recipes through `list_client_integration_recipes` and JSON examples under `examples/`.
- Local MCP config file scanning through `scan_mcp_config_files`.
- Final response review through `review_agent_final_response`.
- Release readiness through `npm run check:v3`, covering typecheck, tests, build, audit, package dry-run inclusion, and `mcp_readiness_report`.
- Hosted API request/response boundaries documented in `docs/hosted-api-shape.md` without adding hosting state.
- Expanded owned V3 fixtures for Expo, Auth0, Stripe, Supabase, and AI SDK boundaries.
- Focused V3 tests for security, artifact scoring, evals, recipes, promotion, and self-contract review.
- Schema hygiene: `src/tools/schemas.ts` is now a thin barrel over focused modules in `src/tools/schemas/`.

## Remaining Future Work

- GitHub-backed memory adapter with repo, branch, and PR workflow.
- Hosted/team memory, analytics, review inbox, and persistence.
- CLI packaging and local command UX.
- Optional typed client SDK helpers around the recipe JSON.

## Decision

Treat V3 as implemented for the local-first MCP core. The next work should be hardening and expanding stack packs, not starting hosting or CLI unless the product direction changes.

The completion audit lives in `docs/v1-v3-completion-audit.md`. V4 scope lives in `docs/v4-scope.md`; the V4 scope audit lives in `docs/v4-scope-audit.md`. The next local-first product-depth scope lives in `docs/v5-scope.md`, followed by local governance maturity in `docs/v6-scope.md`, local strategic intelligence in `docs/v7-scope.md`, and local governance automation in `docs/v8-scope.md`.
