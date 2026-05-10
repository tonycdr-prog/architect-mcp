import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { createV10ImplementationSlicePlan, getV10ProductizationBlueprint, planPrimerDashboard, runV10EvalHarness, validateV10ProductizationBoundary } from "../domain/v10Productization.js";
import { safeJsonResponse } from "./responses.js";
import { v10BlueprintOutputSchema, v10BlueprintRequestSchema, v10BoundaryReviewOutputSchema, v10BoundaryReviewRequestSchema, v10DashboardPlanOutputSchema, v10DashboardPlanRequestSchema, v10EvalHarnessOutputSchema, v10SlicePlanOutputSchema, v10SlicePlanRequestSchema } from "./schemas.js";

export function registerV10Tools(server: McpServer): void {
  server.registerTool("get_v10_productization_blueprint", {
    title: "Get Productization Boundary Blueprint",
    description: "Return the hosted product API, storage, repository, dashboard, and implementation-slice contract for the compatibility productization boundary.",
    inputSchema: { request: v10BlueprintRequestSchema.optional() },
    outputSchema: v10BlueprintOutputSchema
  }, async ({ request }) => safeJsonResponse(() => getV10ProductizationBlueprint(request)));

  server.registerTool("create_v10_implementation_slice_plan", {
    title: "Create Productization Boundary Slice Plan",
    description: "Return productization boundary implementation slices with required MCP pre-edit and post-edit gates.",
    inputSchema: { request: v10SlicePlanRequestSchema.optional() },
    outputSchema: v10SlicePlanOutputSchema
  }, async ({ request }) => safeJsonResponse(() => createV10ImplementationSlicePlan(request)));

  server.registerTool("plan_primer_dashboard", {
    title: "Plan Primer Dashboard",
    description: "Return Primer React dashboard screens, components, data sources, and accessibility checks for the productization boundary.",
    inputSchema: { request: v10DashboardPlanRequestSchema.optional() },
    outputSchema: v10DashboardPlanOutputSchema
  }, async ({ request }) => safeJsonResponse(() => planPrimerDashboard(request)));

  server.registerTool("validate_v10_productization_boundary", {
    title: "Validate Productization Boundary",
    description: "Check hosted API, storage, dashboard, policy, and billing boundaries for productization regressions.",
    inputSchema: { request: v10BoundaryReviewRequestSchema.optional() },
    outputSchema: v10BoundaryReviewOutputSchema
  }, async ({ request }) => safeJsonResponse(() => validateV10ProductizationBoundary(request)));

  server.registerTool("run_v10_eval_harness", {
    title: "Run Productization Boundary Eval Harness",
    description: "Run deterministic productization boundary evals for the compatibility stage.",
    inputSchema: {},
    outputSchema: v10EvalHarnessOutputSchema
  }, async () => safeJsonResponse(() => runV10EvalHarness()));
}
