import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { createV10ImplementationSlicePlan, getV10ProductizationBlueprint, planPrimerDashboard, runV10EvalHarness, validateV10ProductizationBoundary } from "../domain/v10Productization.js";
import { safeJsonResponse } from "./responses.js";
import { genericObjectOutputSchema, v10BlueprintRequestSchema, v10BoundaryReviewRequestSchema, v10DashboardPlanRequestSchema, v10SlicePlanRequestSchema } from "./schemas.js";

export function registerV10Tools(server: McpServer): void {
  server.registerTool("get_v10_productization_blueprint", {
    title: "Get V10 Productization Blueprint",
    description: "Return the hosted product API, storage, repository, dashboard, and implementation-slice contract for V10.",
    inputSchema: { request: v10BlueprintRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => getV10ProductizationBlueprint(request)));

  server.registerTool("create_v10_implementation_slice_plan", {
    title: "Create V10 Implementation Slice Plan",
    description: "Return V10 implementation slices with required MCP pre-edit and post-edit gates.",
    inputSchema: { request: v10SlicePlanRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => createV10ImplementationSlicePlan(request)));

  server.registerTool("plan_primer_dashboard", {
    title: "Plan Primer Dashboard",
    description: "Return Primer React dashboard screens, components, data sources, and accessibility checks for V10.",
    inputSchema: { request: v10DashboardPlanRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => planPrimerDashboard(request)));

  server.registerTool("validate_v10_productization_boundary", {
    title: "Validate V10 Productization Boundary",
    description: "Check V10 hosted API, storage, dashboard, policy, and billing boundaries for productization regressions.",
    inputSchema: { request: v10BoundaryReviewRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => validateV10ProductizationBoundary(request)));

  server.registerTool("run_v10_eval_harness", {
    title: "Run V10 Eval Harness",
    description: "Run deterministic V10 productization boundary evals.",
    inputSchema: {},
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => runV10EvalHarness()));
}
