import type { V10ApiRoute, V10DashboardScreen, V10ImplementationSlice, V10RepositoryBoundary, V10StorageEntity } from "./v10ProductizationTypes.js";

export const v10RepositoryBoundaries: V10RepositoryBoundary[] = [
  {
    name: "OrgRepository",
    owns: ["users", "orgs", "memberships", "invitations"],
    forbiddenConsumers: ["dashboard components", "CLI command handlers"],
    requiredChecks: ["role check", "org_id tenant filter", "audit event"]
  },
  {
    name: "ProjectRepository",
    owns: ["projects", "project_repository_links"],
    forbiddenConsumers: ["dashboard components", "GitHub webhook handlers without org mapping"],
    requiredChecks: ["org_id tenant filter", "role check"]
  },
  {
    name: "ReviewRepository",
    owns: ["review_sessions", "review_artifacts", "review_findings", "review_baselines"],
    forbiddenConsumers: ["dashboard components"],
    requiredChecks: ["no raw repo code by default", "artifact provenance", "org_id tenant filter"]
  },
  {
    name: "PolicyRepository",
    owns: ["policy_bundles", "policy_versions", "policy_rollouts", "policy_exceptions"],
    forbiddenConsumers: ["dashboard components", "remote clients bypassing local repo instructions"],
    requiredChecks: ["source provenance", "rollout mode", "emergency disable", "local instruction conflict check"]
  },
  {
    name: "IntegrationRepository",
    owns: ["github_installations", "github_repositories", "webhook_events"],
    forbiddenConsumers: ["dashboard components"],
    requiredChecks: ["webhook signature", "least privilege scopes", "idempotency"]
  },
  {
    name: "BillingRepository",
    owns: ["billing_customers", "subscriptions", "entitlements", "usage_events"],
    forbiddenConsumers: ["local MCP safety gates", "dashboard components"],
    requiredChecks: ["webhook signature", "event idempotency", "hosted-feature-only gating"]
  }
];

export const v10StorageEntities: V10StorageEntity[] = [
  entity("users", "accounts", false, "OIDC-linked user identity records.", ["email"], "account lifetime"),
  entity("orgs", "accounts", true, "Organization root records.", [], "account lifetime"),
  entity("memberships", "accounts", true, "Role membership for teams.", [], "account lifetime"),
  entity("invitations", "accounts", true, "Pending team invitations.", ["email"], "30 days"),
  entity("projects", "product-api", true, "Hosted project boundary for reviews and policies.", [], "account lifetime"),
  entity("review_sessions", "storage", true, "Stored review run metadata and normalized outputs.", [], "plan retention"),
  entity("review_artifacts", "storage", true, "Explicitly supplied summaries, reports, hashes, and provenance.", ["artifact_ref"], "plan retention"),
  entity("review_findings", "storage", true, "Stable finding records for history and baselines.", [], "plan retention"),
  entity("review_baselines", "storage", true, "Accepted baseline finding snapshots.", [], "account lifetime"),
  entity("policy_bundles", "remote-policy", true, "Policy bundle identities.", [], "account lifetime"),
  entity("policy_versions", "remote-policy", true, "Immutable policy versions with source provenance.", ["signed_manifest"], "account lifetime"),
  entity("policy_rollouts", "remote-policy", true, "Rollout state for suggest/warn/block modes.", [], "account lifetime"),
  entity("policy_exceptions", "remote-policy", true, "Explicit policy exceptions with review context.", [], "account lifetime"),
  entity("github_installations", "github", true, "GitHub App installation mapping.", ["installation_id"], "while connected"),
  entity("github_repositories", "github", true, "Repository links and permissions metadata.", [], "while connected"),
  entity("billing_customers", "billing", true, "Stripe customer references.", ["stripe_customer_id"], "billing lifetime"),
  entity("subscriptions", "billing", true, "Plan and subscription status.", ["stripe_subscription_id"], "billing lifetime"),
  entity("entitlements", "billing", true, "Resolved hosted feature gates.", [], "billing lifetime"),
  entity("usage_events", "billing", true, "Usage metering and quota decisions.", [], "plan retention"),
  entity("audit_events", "accounts", true, "Security and admin audit trail.", ["actor_id", "target_id"], "1 year")
];

export const v10ApiRoutes: V10ApiRoute[] = [
  route("GET", "/v1/orgs", "accounts", "user", true, "OrgRepository", ["List orgs the current user can access."]),
  route("POST", "/v1/orgs", "accounts", "user", true, "OrgRepository", ["Create an org and owner membership."], true),
  route("GET", "/v1/orgs/:orgId/members", "accounts", "org-role", true, "OrgRepository", ["List members by role."]),
  route("POST", "/v1/orgs/:orgId/invitations", "accounts", "org-role", true, "OrgRepository", ["Invite a member."], true),
  route("GET", "/v1/orgs/:orgId/projects", "product-api", "org-role", true, "ProjectRepository", ["List projects."]),
  route("POST", "/v1/orgs/:orgId/projects", "product-api", "org-role", true, "ProjectRepository", ["Create project."], true),
  route("POST", "/v1/orgs/:orgId/projects/:projectId/reviews", "storage", "org-role", true, "ReviewRepository", ["Persist normalized review output without raw repo code by default."], true),
  route("GET", "/v1/orgs/:orgId/projects/:projectId/reviews/:reviewId", "storage", "org-role", true, "ReviewRepository", ["Read stored review report."]),
  route("GET", "/v1/orgs/:orgId/policies", "remote-policy", "org-role", true, "PolicyRepository", ["List policy bundles."]),
  route("POST", "/v1/orgs/:orgId/policies", "remote-policy", "org-role", true, "PolicyRepository", ["Create policy bundle draft."], true),
  route("POST", "/v1/orgs/:orgId/policies/:policyId/versions", "remote-policy", "org-role", true, "PolicyRepository", ["Create immutable signed policy version."], true),
  route("POST", "/v1/orgs/:orgId/policies/:policyId/preview", "remote-policy", "org-role", true, "PolicyRepository", ["Preview rollout against supplied findings."]),
  route("PATCH", "/v1/orgs/:orgId/policies/:policyId/rollout", "remote-policy", "org-role", true, "PolicyRepository", ["Set suggest/warn/block rollout mode."]),
  route("GET", "/v1/orgs/:orgId/github/installations", "github", "org-role", true, "IntegrationRepository", ["List GitHub App installations."]),
  route("POST", "/v1/github/webhook", "github", "webhook-signature", false, "IntegrationRepository", ["Verify GitHub webhook signature before processing."], true),
  route("POST", "/v1/orgs/:orgId/billing/checkout", "billing", "org-role", true, "BillingRepository", ["Create Stripe checkout session."], true),
  route("POST", "/v1/orgs/:orgId/billing/portal", "billing", "org-role", true, "BillingRepository", ["Create Stripe customer portal session."], true),
  route("POST", "/v1/billing/webhook", "billing", "webhook-signature", false, "BillingRepository", ["Verify Stripe webhook signature and idempotency."], true),
  route("GET", "/v1/orgs/:orgId/audit-events", "accounts", "org-role", true, "OrgRepository", ["Read tenant-scoped audit events."])
];

export const v10DashboardScreens: V10DashboardScreen[] = [
  screen("/orgs/:orgId", "Org overview", ["PageLayout", "PageHeader", "NavList", "DataTable", "StateLabel"], ["HostedApi.orgs", "HostedApi.projects"], ["semantic heading order", "keyboard navigation"]),
  screen("/orgs/:orgId/projects/:projectId", "Project dashboard", ["PageLayout", "UnderlineNav", "DataTable", "Label", "ActionMenu"], ["HostedApi.projects", "HostedApi.reviews"], ["table headers", "descriptive action labels"]),
  screen("/orgs/:orgId/projects/:projectId/reviews/:reviewId", "Review report", ["PageLayout", "Timeline", "Label", "Flash", "Button"], ["HostedApi.reviews"], ["status not conveyed by color alone", "focus error summary"]),
  screen("/orgs/:orgId/policies", "Policy library", ["PageLayout", "DataTable", "ActionMenu", "Dialog", "Button"], ["HostedApi.policies"], ["dialog labelled title", "keyboard dismissal"]),
  screen("/orgs/:orgId/policies/:policyId", "Policy detail", ["PageLayout", "SegmentedControl", "DataTable", "ConfirmationDialog"], ["HostedApi.policies"], ["destructive action confirmation"]),
  screen("/orgs/:orgId/github", "GitHub connections", ["PageLayout", "Banner", "DataTable", "Button"], ["HostedApi.github"], ["clear OAuth/install labels"]),
  screen("/orgs/:orgId/team", "Team settings", ["PageLayout", "DataTable", "SelectPanel", "TextInput"], ["HostedApi.orgs"], ["form labels", "invitation error focus"]),
  screen("/orgs/:orgId/billing", "Usage and billing", ["PageLayout", "ProgressBar", "DataTable", "Button"], ["HostedApi.billing"], ["quota text alongside progress color"]),
  screen("/orgs/:orgId/audit", "Audit log", ["PageLayout", "DataTable", "Label", "RelativeTime"], ["HostedApi.audit"], ["time text readable without hover"])
];

export const v10ImplementationSlices: V10ImplementationSlice[] = [
  slice("v10-01-product-skeleton", "Product architecture skeleton", ["product-api", "storage", "mcp-loop"], ["src/domain/v10Productization*.ts", "src/tools/v10Tools.ts", "docs/v10-architecture-contract.md"], ["npm test", "npm run check:v10"]),
  slice("v10-02-storage", "Postgres schema and repositories", ["storage"], ["src/product/repositories/*", "migrations/*"], ["npm test", "migration dry-run"]),
  slice("v10-03-accounts", "Accounts, teams, roles, and audit events", ["accounts"], ["src/product/accounts/*"], ["npm test", "tenant isolation tests"]),
  slice("v10-04-dashboard-shell", "Primer dashboard shell", ["dashboard"], ["dashboard/src/*"], ["dashboard test", "accessibility checks"]),
  slice("v10-05-review-history", "Hosted review persistence", ["storage", "product-api"], ["src/product/reviews/*"], ["npm test", "review upload contract tests"]),
  slice("v10-06-remote-policy", "Remote policy runtime", ["remote-policy"], ["src/product/policies/*"], ["npm test", "policy conflict tests"]),
  slice("v10-07-github", "GitHub App adapter", ["github"], ["src/product/github/*"], ["webhook signature tests"]),
  slice("v10-08-billing", "Stripe billing and entitlements", ["billing"], ["src/product/billing/*"], ["Stripe webhook idempotency tests"]),
  slice("v10-09-cli-sdk", "CLI and typed SDK", ["cli", "sdk"], ["src/cli/*", "src/sdk/*"], ["SDK contract tests", "CLI mocked API tests"]),
  slice("v10-10-hardening", "Security, docs, and smoke tests", ["mcp-loop"], ["docs/*", "examples/*", "tests/*"], ["npm run check:v10", "package dry-run"])
];

function entity(table: string, area: V10StorageEntity["area"], orgScoped: boolean, purpose: string, sensitiveColumns: string[], retention: string): V10StorageEntity {
  return { table, area, orgScoped, purpose, sensitiveColumns, retention };
}

function route(method: V10ApiRoute["method"], path: string, area: V10ApiRoute["area"], auth: V10ApiRoute["auth"], tenantScoped: boolean, repositoryBoundary: string, notes: string[], idempotency = false): V10ApiRoute {
  return { method, path, area, auth, tenantScoped, repositoryBoundary, notes, idempotency };
}

function screen(routePath: string, title: string, primerComponents: string[], dataSources: string[], accessibilityChecks: string[]): V10DashboardScreen {
  return { route: routePath, title, primerComponents, dataSources, accessibilityChecks };
}

function slice(id: string, title: string, areas: V10ImplementationSlice["areas"], likelyFiles: string[], verification: string[]): V10ImplementationSlice {
  return {
    id,
    title,
    areas,
    likelyFiles,
    verification,
    mcpToolsBefore: ["interpret_implementation_intent", "load_triggered_stack_guidance", "create_pre_edit_contract", "review_proposed_file_plan"],
    mcpToolsAfter: ["review_implementation_against_contract", "review_repo_structure", "review_agent_session", "score_agent_artifacts", "mcp_readiness_report"],
    stopCondition: "Fix all MCP findings and failing verification before starting the next V10 slice."
  };
}
