import { v10ApiRoutes, v10DashboardScreens, v10ImplementationSlices, v10RepositoryBoundaries, v10StorageEntities } from "./v10ProductizationCatalog.js";
import type { V10Finding, V10ProductArea, V10ReadinessStatus, V10RolloutMode } from "./v10ProductizationTypes.js";

export type V10BlueprintRequest = {
  area?: V10ProductArea | "all";
};

export type V10BoundaryReviewRequest = {
  routes?: Array<{ path: string; tenantScoped?: boolean; repositoryBoundary?: string; auth?: string }>;
  storageEntities?: Array<{ table: string; orgScoped?: boolean; purpose?: string }>;
  dashboardScreens?: Array<{ route: string; primerComponents?: string[]; dataSources?: string[] }>;
  policyRollouts?: Array<{ mode?: string; preservesLocalInstructions?: boolean; hasEmergencyDisable?: boolean }>;
  billingGates?: Array<{ feature: string; gatesLocalMcp?: boolean }>;
};

export function getV10ProductizationBlueprint(input: V10BlueprintRequest = {}) {
  const area = input.area ?? "all";
  return {
    version: "v10",
    status: "implementation-contract",
    principle: "Hosted product features wrap the local-first MCP core; they do not replace or weaken it.",
    routes: filterByArea(v10ApiRoutes, area),
    storage: filterByArea(v10StorageEntities, area),
    repositoryBoundaries: v10RepositoryBoundaries.filter((boundary) => area === "all" || boundary.owns.some((owned) => filterByArea(v10StorageEntities, area).some((entity) => entity.table === owned))),
    dashboardScreens: area === "all" || area === "dashboard" ? v10DashboardScreens : [],
    implementationSlices: v10ImplementationSlices.filter((slice) => area === "all" || slice.areas.includes(area)),
    nonNegotiables: [
      "No raw repo code persistence by default.",
      "Every org-scoped route and table must enforce org_id tenant boundaries.",
      "Dashboard components consume API/SDK surfaces, never repositories directly.",
      "Billing gates hosted convenience features only, never local MCP safety checks.",
      "Remote policy conflicts with repo-local instructions must be surfaced.",
      "Every implementation slice runs the MCP pre-edit and post-edit loop."
    ]
  };
}

export function createV10ImplementationSlicePlan(input: { sliceId?: string } = {}) {
  const selected = input.sliceId ? v10ImplementationSlices.filter((slice) => slice.id === input.sliceId) : v10ImplementationSlices;
  return {
    slices: selected,
    warnings: input.sliceId && selected.length === 0 ? [`No V10 implementation slice matched ${input.sliceId}.`] : [],
    mcpLoopRequired: true,
    beforeEverySlice: ["interpret_implementation_intent", "load_triggered_stack_guidance", "create_pre_edit_contract", "review_proposed_file_plan"],
    afterEverySlice: ["review_implementation_against_contract", "review_repo_structure", "review_agent_session", "score_agent_artifacts", "mcp_readiness_report"],
    stopRule: "Do not start the next slice until MCP findings and verification failures from the current slice are fixed."
  };
}

export function planPrimerDashboard(input: { screenRoute?: string } = {}) {
  const screens = input.screenRoute ? v10DashboardScreens.filter((screen) => screen.route === input.screenRoute) : v10DashboardScreens;
  return {
    primerSource: "Primer MCP guidance",
    constraints: [
      "Use Primer React components first.",
      "Use CSS Modules for custom styling.",
      "Do not use sx for styling.",
      "Do not use Box for styling.",
      "Use Primer design tokens instead of hard-coded colors.",
      "Button labels must describe the action without relying on icons.",
      "Loading and error states must preserve keyboard focus and announce status."
    ],
    screens,
    warnings: input.screenRoute && screens.length === 0 ? [`No V10 dashboard screen matched ${input.screenRoute}.`] : [],
    sharedLayout: {
      shell: ["PageLayout", "PageHeader", "NavList"],
      status: ["Label", "StateLabel", "Banner", "Flash"],
      denseData: ["DataTable", "ActionMenu", "Pagination"],
      forms: ["FormControl", "TextInput", "Textarea", "SelectPanel", "Button"],
      confirmation: ["Dialog", "ConfirmationDialog"]
    }
  };
}

export function validateV10ProductizationBoundary(input: V10BoundaryReviewRequest = {}) {
  const findings: V10Finding[] = [];
  for (const route of input.routes ?? []) {
    if (route.path.includes(":orgId") && route.tenantScoped !== true) {
      findings.push(finding("V10_ROUTE_TENANT_SCOPE", "fail", "product-api", `${route.path} includes orgId but is not tenant-scoped.`, "Require org_id authorization before repository access."));
    }
    if (!route.repositoryBoundary) {
      findings.push(finding("V10_ROUTE_REPOSITORY_BOUNDARY", "fail", "product-api", `${route.path} has no server-owned repository boundary.`, "Route handlers must call service/repository modules, not storage directly."));
    }
    if (route.auth === "public" && route.path.includes(":orgId")) {
      findings.push(finding("V10_PUBLIC_ORG_ROUTE", "fail", "accounts", `${route.path} is org-scoped but public.`, "Require user auth plus org role checks."));
    }
  }
  for (const entity of input.storageEntities ?? []) {
    if (entity.table !== "users" && entity.orgScoped !== true) {
      findings.push(finding("V10_STORAGE_TENANT_SCOPE", "fail", "storage", `${entity.table} is product data without org scope.`, "Add org_id or document why the entity is global infrastructure metadata."));
    }
    if (/raw repo code|source files/i.test(entity.purpose ?? "")) {
      findings.push(finding("V10_RAW_REPO_CODE_STORAGE", "fail", "storage", `${entity.table} appears to persist raw repo code.`, "Persist summaries, findings, hashes, and explicit artifacts only by default."));
    }
  }
  for (const screen of input.dashboardScreens ?? []) {
    if ((screen.dataSources ?? []).some((source) => /Repository$/.test(source))) {
      findings.push(finding("V10_DASHBOARD_REPOSITORY_LEAK", "fail", "dashboard", `${screen.route} reads a repository boundary directly.`, "Dashboard screens must use API or SDK data sources."));
    }
    if ((screen.primerComponents ?? []).includes("Box")) {
      findings.push(finding("V10_PRIMER_BOX_STYLING", "warn", "dashboard", `${screen.route} uses Box in its component plan.`, "Follow Primer MCP guidance: use CSS Modules for custom styling instead of Box styling."));
    }
  }
  for (const rollout of input.policyRollouts ?? []) {
    if (!isRolloutMode(rollout.mode)) {
      findings.push(finding("V10_POLICY_ROLLOUT_MODE", "fail", "remote-policy", `Unsupported rollout mode ${rollout.mode ?? "missing"}.`, "Use suggest, warn, or block."));
    }
    if (rollout.preservesLocalInstructions !== true) {
      findings.push(finding("V10_POLICY_LOCAL_CONFLICT", "fail", "remote-policy", "Remote policy can override local repo instructions silently.", "Surface the conflict and require explicit resolution."));
    }
    if (rollout.hasEmergencyDisable !== true) {
      findings.push(finding("V10_POLICY_NO_DISABLE", "warn", "remote-policy", "Policy rollout lacks emergency disable.", "Add an emergency disable or rollback path."));
    }
  }
  for (const gate of input.billingGates ?? []) {
    if (gate.gatesLocalMcp) {
      findings.push(finding("V10_BILLING_GATES_LOCAL_MCP", "fail", "billing", `${gate.feature} gates local MCP behavior.`, "Billing may gate hosted convenience features only."));
    }
  }
  return {
    status: statusFromFindings(findings),
    findings,
    summary: {
      failures: findings.filter((item) => item.status === "fail").length,
      warnings: findings.filter((item) => item.status === "warn").length
    }
  };
}

export function runV10EvalHarness() {
  const cases = [
    {
      name: "blueprint exposes product routes and repository boundaries",
      passed: getV10ProductizationBlueprint().routes.length >= 10 && getV10ProductizationBlueprint().repositoryBoundaries.length >= 5
    },
    {
      name: "storage entities avoid raw repo code by default",
      passed: !v10StorageEntities.some((entity) => /raw repo code|source files/i.test(entity.purpose))
    },
    {
      name: "Primer dashboard follows MCP component constraints",
      passed: planPrimerDashboard().constraints.some((constraint) => /Do not use sx/.test(constraint))
    },
    {
      name: "boundary review fails raw code persistence",
      passed: validateV10ProductizationBoundary({ storageEntities: [{ table: "repo_files", orgScoped: true, purpose: "Store raw repo code" }] }).status === "fail"
    },
    {
      name: "billing cannot gate local MCP safety",
      passed: validateV10ProductizationBoundary({ billingGates: [{ feature: "local safety checks", gatesLocalMcp: true }] }).status === "fail"
    },
    {
      name: "implementation slices require MCP loop",
      passed: v10ImplementationSlices.every((slice) => slice.mcpToolsBefore.includes("create_pre_edit_contract") && slice.mcpToolsAfter.includes("review_agent_session"))
    }
  ];
  return {
    stage: "v10",
    status: cases.every((testCase) => testCase.passed) ? "pass" : "fail",
    summary: {
      total: cases.length,
      passed: cases.filter((testCase) => testCase.passed).length,
      failed: cases.filter((testCase) => !testCase.passed).length
    },
    results: cases
  };
}

function filterByArea<T extends { area: V10ProductArea }>(items: T[], area: V10ProductArea | "all"): T[] {
  return area === "all" ? items : items.filter((item) => item.area === area);
}

function finding(code: string, status: V10ReadinessStatus, area: V10ProductArea, message: string, recommendation: string): V10Finding {
  return { code, status, area, message, recommendation };
}

function statusFromFindings(findings: V10Finding[]): V10ReadinessStatus {
  if (findings.some((item) => item.status === "fail")) return "fail";
  if (findings.length) return "warn";
  return "pass";
}

function isRolloutMode(mode: string | undefined): mode is V10RolloutMode {
  return mode === "suggest" || mode === "warn" || mode === "block";
}
