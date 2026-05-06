# V10 Productization Implementation

V10 turns the previous productization concepts into a concrete hosted-product contract while preserving the local-first MCP core.

The implementation is intentionally boundary-first:

- `/mcp` stays stateless and compatible.
- Hosted product APIs live under `/v1/*`.
- Dashboard code must consume API or SDK data, never repositories directly.
- Database access belongs to server-owned repository modules.
- Billing gates hosted convenience features only.
- Remote policy rollouts cannot silently override repo-local instructions.
- Every implementation slice must use architect-mcp before and after edits.

## Implemented V10 MCP Surface

- `get_v10_productization_blueprint`: returns route, storage, repository, dashboard, and staged-slice contracts.
- `create_v10_implementation_slice_plan`: returns the ten V10 implementation slices and required MCP tools for each slice.
- `plan_primer_dashboard`: returns Primer React screen/component/accessibility plans based on Primer MCP guidance.
- `validate_v10_productization_boundary`: checks route, storage, dashboard, policy, and billing boundary regressions.
- `run_v10_eval_harness`: runs deterministic V10 boundary evals.

## Product API Boundary

The route catalog defines authenticated `/v1/*` endpoints for orgs, projects, reviews, policies, GitHub, billing, usage, and audit events. Every route has a named repository boundary, tenant-scope expectation, auth mode, and idempotency marker where relevant.

Stateful implementation should be added behind these repository boundaries:

- `OrgRepository`
- `ProjectRepository`
- `ReviewRepository`
- `PolicyRepository`
- `IntegrationRepository`
- `BillingRepository`

## Storage Boundary

The storage catalog covers users, orgs, memberships, invitations, projects, review sessions, artifacts, findings, baselines, policy bundles, policy versions, policy rollouts, exceptions, GitHub installations, billing records, usage events, and audit events.

Raw repository code must not be persisted by default. Store summaries, normalized findings, hashes, provenance, and explicitly supplied artifacts.

## Primer Dashboard Boundary

Dashboard implementation should use Primer React components first:

- `PageLayout`, `PageHeader`, `NavList`, `UnderlineNav`
- `DataTable`, `ActionMenu`, `Pagination`
- `Label`, `StateLabel`, `Banner`, `Flash`
- `FormControl`, `TextInput`, `Textarea`, `SelectPanel`, `Button`
- `Dialog`, `ConfirmationDialog`

Primer MCP constraints apply:

- Use CSS Modules for custom styling.
- Do not use `sx` for styling.
- Do not use `Box` for styling.
- Use Primer design tokens instead of hard-coded colors.
- Buttons need clear labels.
- Loading and error states must preserve focus and announce status.

## MCP-Gated Slice Loop

Before every V10 slice:

1. `interpret_implementation_intent`
2. `load_triggered_stack_guidance`
3. `create_pre_edit_contract`
4. `review_proposed_file_plan`

After every V10 slice:

1. `review_implementation_against_contract`
2. `review_repo_structure`
3. `review_agent_session`
4. `score_agent_artifacts`
5. `mcp_readiness_report`
6. `npm run check:v10`

Do not start the next slice until findings and failed checks are fixed.
