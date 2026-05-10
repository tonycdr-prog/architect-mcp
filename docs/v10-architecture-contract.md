# Productization Boundary Architecture Contract

## Goal

Add hosted productization around architect-mcp without weakening the local-first MCP core.

## Required Boundaries

- MCP domain logic remains pure and reusable without hosted state.
- Product API routes use server-owned service and repository modules.
- Dashboard screens consume API or SDK types only.
- Product storage is tenant-scoped by `org_id` except global infrastructure metadata.
- Remote policies preserve source provenance and surface conflicts with repo-local instructions.
- Billing entitlements gate hosted convenience features only.
- GitHub integration is least-privilege and webhook-signature verified.

## Required Artifacts

- Productization boundary blueprint.
- Productization boundary slice plan.
- Primer dashboard plan.
- Productization boundary review.
- Productization boundary eval harness.
- Staged readiness script.

## Verification

- `npm run typecheck`
- `npm test`
- `npm run build`
- `npm run check:v10`

## Stop Conditions

- Any productization boundary finding with status `fail`.
- Any implementation slice missing MCP pre-edit or post-edit review.
- Any route without a repository boundary.
- Any org-scoped route or table without tenant-scope enforcement.
- Any dashboard plan that reads repository modules directly.
- Any billing gate that blocks local MCP safety behavior.
