# V10 Productization Concepts

V10 is a concept exploration for productized architect-mcp capabilities after the local MCP core, operating model, and backend architecture are concrete.

This is not an implementation plan. It sketches what billing, accounts, teams, dashboards, and remote policy management could look like when the project is ready to move beyond local-first MCP behavior.

## V10 Direction

V10 explores a hosted product layer around the local MCP:

- Accounts and identity.
- Organizations and teams.
- Billing and plan boundaries.
- Product dashboards.
- Remote policy management.
- Shared standards governance.
- Usage, value, and trust reporting.

## Product Principles

- The local MCP remains useful without an account.
- Hosted features should remove coordination overhead, not slow down YOLO-style agent work.
- Paid features should map to team value: shared standards, auditability, repeatable governance, and easier onboarding.
- The product should show evidence and decisions, not become a noisy dashboard.
- Remote policy management should never silently override repo-local contracts or current user instructions.
- Sensitive code, secrets, and repo details need explicit boundaries before any hosted processing exists.

## Account Concepts

Goal: give individuals a lightweight identity layer only when they need sync, billing, teams, or remote policy.

Possible capabilities:

- Personal account with workspace preferences.
- Local-first onboarding that explains what still works without sign-in.
- Connected MCP clients and environments.
- Personal defaults for YOLO budget, output verbosity, evidence strictness, and preferred standards profiles.
- Exportable account data and clear deletion controls.

Key screens:

- Account overview.
- Connected clients.
- Preferences.
- Security and sessions.
- Data export and deletion.

## Team Concepts

Goal: let teams share standards and governance without forcing every repo into one rigid setup.

Possible capabilities:

- Organizations with teams, roles, and policy ownership.
- Team-level standards profiles: minimal, balanced, strict, regulated, migration, frontend-heavy, backend-heavy.
- Repo/project groups without requiring a GitHub integration in the first version.
- Team invitations and role-based permissions.
- Shared onboarding instructions for agents and humans.

Roles:

- Owner: billing, team settings, policy publishing.
- Maintainer: policy authoring, approval, rollout.
- Developer: consumes policies and review reports.
- Viewer: reads reports and dashboards.

## Billing Concepts

Goal: price the hosted layer around coordination value, not local MCP usage alone.

Possible plan shapes:

- Free: local MCP, docs, local policy packs, local evals.
- Pro: personal hosted account, saved preferences, private policy library, richer reports.
- Team: shared policy management, team dashboards, governance packs, member controls.
- Enterprise: SSO, audit exports, advanced retention controls, custom policy packs, support.

Billing dimensions to evaluate:

- Seat count.
- Active projects.
- Remote policy libraries.
- Hosted report retention.
- Advanced eval/report volume.

Avoid early:

- Charging for local-only MCP behavior.
- Complicated usage pricing before value is obvious.
- Billing gates inside pre-edit safety checks.

## Dashboard Concepts

Goal: show where agent work is safe, blocked, noisy, or improving.

Useful dashboard views:

- Standards coverage: which policies and packs are active by project.
- Risk overview: auth, security, dependency, architecture, verification, and documentation risks.
- Agent behavior: skipped checks, scope drift, missing evidence, repeated assumptions.
- Policy health: noisy rules, weak rules, stale rules, untested rules.
- Scenario outcomes: pass/fail trends for fresh app, feature, bug fix, refactor, docs, and security-sensitive runs.
- Team adoption: projects using current standards versus older versions.

Dashboard anti-goals:

- Vanity metrics.
- Endless charts without action.
- Hiding the evidence behind scores.
- Turning every warning into management noise.

## Remote Policy Management Concepts

Goal: let teams publish, version, test, and roll out shared standards while preserving local repo autonomy.

Possible capabilities:

- Remote policy libraries with versioned bundles.
- Draft, review, trial, adopt, deprecated, and retired policy states.
- Rollout modes: suggest only, warn, block.
- Compatibility checks against local contracts and stack packs.
- Policy diff summaries in plain English.
- Local preview before adoption.
- Signed policy bundle manifests.
- Emergency disable for noisy or broken policies.

Important rule:

Remote policy should be additive and explicit. The client should show which remote policy was applied, why, and how to ignore or override it locally when appropriate.

## Governance Concepts

Goal: make shared standards trustworthy enough for teams without making agents timid.

Possible capabilities:

- Policy approval workflow.
- Rule ownership and review cadence.
- Source provenance and evidence requirements.
- Eval coverage before policy promotion.
- Noise score and rollout risk.
- Exception process with expiry and rationale.
- Human-readable policy pack pages.

## Trust And Safety Concepts

Goal: make hosted behavior safe enough before any code or repo context leaves the local environment.

Questions to answer before implementation:

- What data is sent remotely?
- What stays local always?
- How are secrets detected and redacted?
- How long are reports retained?
- Who can view reports?
- Can teams require local-only mode for sensitive projects?
- How are remote policies authenticated and integrity-checked?
- How does a user know whether a finding came from local rules, remote policy, or source-backed stack guidance?

## V10 Concept Screens

Potential screens for product exploration:

- Account setup: "Use local only" versus "Create account for sync and teams".
- Organization dashboard: project health, standards coverage, agent behavior, policy health.
- Policy library: bundles, versions, source provenance, eval coverage, rollout state.
- Policy detail: rules, examples, detectors, source evidence, rollout impact, exceptions.
- Team settings: members, roles, defaults, security.
- Billing: plan, seats, usage/value summary, invoices.
- Project dashboard: current standards, latest local report uploads, risks, suggested next action.
- Remote policy preview: what would change before adoption.

## V10 Concept Risks

- Productization could distract from making the MCP core excellent.
- Dashboards could create management theater instead of better agent outcomes.
- Billing could push the product toward lock-in if local value is weakened.
- Remote policies could become dangerous if they silently override local context.
- Stored reports could create privacy, security, and retention obligations.
- Team workflows could slow users down if every policy change requires heavy review.

## V10 Concept Evaluation

Before implementation, validate:

- Users understand what remains local.
- Teams can see value without connecting GitHub or uploading code.
- Billing maps to clear team coordination value.
- Remote policy preview prevents surprise enforcement.
- Dashboard screens lead to a next action within one click.
- Novice users can understand policy findings without reading governance jargon.
- Advanced users can inspect source provenance and rule evidence.

## Non-Goals For This Concept Phase

- No hosted service implementation.
- No database schema.
- No billing provider integration.
- No account/auth implementation.
- No team permission implementation.
- No dashboard frontend implementation.
- No remote policy runtime implementation.
- No GitHub integration.

This document is a product concept map only.
