import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";
import { createV10ImplementationSlicePlan, getV10ProductizationBlueprint, planPrimerDashboard, runV10EvalHarness, validateV10ProductizationBoundary } from "../src/domain/v10Productization.js";

describe("V10 productization implementation contract", () => {
  it("defines concrete hosted product routes behind repository boundaries", () => {
    const blueprint = getV10ProductizationBlueprint();

    assert.equal(blueprint.version, "v10");
    assert.equal(blueprint.routes.some((route) => route.path === "/v1/orgs"), true);
    assert.equal(blueprint.routes.some((route) => route.path === "/v1/github/webhook" && route.auth === "webhook-signature"), true);
    assert.equal(blueprint.routes.every((route) => route.repositoryBoundary.length > 0), true);
    assert.equal(blueprint.nonNegotiables.some((rule) => /No raw repo code/.test(rule)), true);
  });

  it("keeps storage tenant-scoped and explicit about retention", () => {
    const blueprint = getV10ProductizationBlueprint({ area: "storage" });

    assert.equal(blueprint.storage.some((entity) => entity.table === "review_sessions"), true);
    assert.equal(blueprint.storage.every((entity) => entity.retention.length > 0), true);
    assert.equal(blueprint.storage.some((entity) => /raw repo code/i.test(entity.purpose)), false);
  });

  it("plans Primer dashboard screens with Primer MCP constraints", () => {
    const plan = planPrimerDashboard();

    assert.equal(plan.constraints.some((constraint) => constraint === "Do not use sx for styling."), true);
    assert.equal(plan.constraints.some((constraint) => constraint === "Do not use Box for styling."), true);
    assert.equal(plan.screens.some((screen) => screen.route === "/orgs/:orgId/billing"), true);
    assert.equal(plan.screens.every((screen) => !screen.dataSources.some((source) => /Repository$/.test(source))), true);
    assert.equal(plan.sharedLayout.denseData.includes("DataTable"), true);
  });

  it("requires every implementation slice to run the MCP loop", () => {
    const plan = createV10ImplementationSlicePlan();

    assert.equal(plan.mcpLoopRequired, true);
    assert.equal(plan.slices.length, 10);
    assert.equal(plan.slices.every((slice) => slice.mcpToolsBefore.includes("review_proposed_file_plan")), true);
    assert.equal(plan.slices.every((slice) => slice.mcpToolsAfter.includes("review_agent_session")), true);
  });

  it("warns when V10 filters do not match anything", () => {
    assert.equal(createV10ImplementationSlicePlan({ sliceId: "missing-slice" }).warnings.length, 1);
    assert.equal(planPrimerDashboard({ screenRoute: "/missing" }).warnings.length, 1);
  });

  it("flags productization boundary regressions", () => {
    const review = validateV10ProductizationBoundary({
      routes: [{ path: "/v1/orgs/:orgId/projects", tenantScoped: false, auth: "public" }],
      storageEntities: [{ table: "repo_files", orgScoped: true, purpose: "Store raw repo code" }],
      dashboardScreens: [{ route: "/orgs/:orgId", dataSources: ["OrgRepository"], primerComponents: ["Box"] }],
      policyRollouts: [{ mode: "force", preservesLocalInstructions: false, hasEmergencyDisable: false }],
      billingGates: [{ feature: "local MCP checks", gatesLocalMcp: true }]
    });

    assert.equal(review.status, "fail");
    assert.equal(review.findings.some((finding) => finding.code === "V10_RAW_REPO_CODE_STORAGE"), true);
    assert.equal(review.findings.some((finding) => finding.code === "V10_BILLING_GATES_LOCAL_MCP"), true);
    assert.equal(review.findings.some((finding) => finding.code === "V10_PRIMER_BOX_STYLING"), true);
  });

  it("requires explicit tenant and policy safety evidence", () => {
    const review = validateV10ProductizationBoundary({
      routes: [{ path: "/v1/orgs/:orgId/projects", repositoryBoundary: "ProjectRepository", auth: "org-role" }],
      storageEntities: [{ table: "projects", purpose: "Project records." }],
      policyRollouts: [{ mode: "warn" }]
    });

    assert.equal(review.status, "fail");
    assert.equal(review.findings.some((finding) => finding.code === "V10_ROUTE_TENANT_SCOPE"), true);
    assert.equal(review.findings.some((finding) => finding.code === "V10_STORAGE_TENANT_SCOPE"), true);
    assert.equal(review.findings.some((finding) => finding.code === "V10_POLICY_LOCAL_CONFLICT"), true);
  });

  it("makes the generated dashboard plan pass the dashboard boundary validator", () => {
    const review = validateV10ProductizationBoundary({
      dashboardScreens: planPrimerDashboard().screens
    });

    assert.equal(review.status, "pass");
  });

  it("runs the V10 eval harness", () => {
    const report = runV10EvalHarness();

    assert.equal(report.status, "pass");
    assert.equal(report.summary.failed, 0);
  });

  it("exposes V10 tools through MCP", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      for (const name of ["get_v10_productization_blueprint", "create_v10_implementation_slice_plan", "plan_primer_dashboard", "validate_v10_productization_boundary", "run_v10_eval_harness"]) {
        assert.equal(tools.tools.some((tool) => tool.name === name), true, `${name} missing`);
      }

      assert.equal((await callJson(client, "get_v10_productization_blueprint", { request: { area: "billing" } })).routes.some((route: { path: string }) => route.path === "/v1/billing/webhook"), true);
      assert.equal((await callJson(client, "create_v10_implementation_slice_plan", { request: { sliceId: "v10-04-dashboard-shell" } })).slices[0].areas.includes("dashboard"), true);
      assert.equal((await callJson(client, "plan_primer_dashboard", { request: { screenRoute: "/orgs/:orgId/team" } })).screens[0].title, "Team settings");
      assert.equal((await callJson(client, "validate_v10_productization_boundary", { request: { billingGates: [{ feature: "local MCP", gatesLocalMcp: true }] } })).status, "fail");
      assert.equal((await callJson(client, "run_v10_eval_harness", {})).status, "pass");
    } finally {
      await close();
    }
  });
});

async function connectTestClient() {
  const server = createArchitectServer();
  const client = new Client({ name: "architect-mcp-v10-test-client", version: "0.1.0" });
  const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
  await Promise.all([
    client.connect(clientTransport),
    server.connect(serverTransport)
  ]);

  return {
    client,
    close: async () => {
      await client.close();
      await server.close();
    }
  };
}

async function callJson(client: Client, name: string, args: Record<string, unknown>) {
  const response = await client.callTool({ name, arguments: args });
  const text = response.content?.[0]?.type === "text" ? response.content[0].text : "{}";
  return JSON.parse(text);
}
