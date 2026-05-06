import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { auditGeneratedRepoQuality, buildQualityRequirementsProfile, evaluateRepoPlanQuality, runRepoQualityEvalScenarios, suggestQualityFollowUpQuestions } from "../domain/repoQualityEval.js";
import { safeJsonResponse } from "./responses.js";
import { qualityFollowUpOutputSchema, qualityRequirementsInputSchema, qualityRequirementsProfileSchema, repoQualityEvalScenariosOutputSchema, repoQualityEvaluationInputSchema, repoQualityEvaluationOutputSchema } from "./schemas.js";

export function registerRepoQualityEvalTools(server: McpServer): void {
  server.registerTool("build_quality_requirements_profile", {
    title: "Build Quality Requirements Profile",
    description: "Turn interview answers into a quality requirements profile with missing questions and confidence.",
    inputSchema: { request: qualityRequirementsInputSchema.optional() },
    outputSchema: qualityRequirementsProfileSchema
  }, async ({ request }) => safeJsonResponse(() => buildQualityRequirementsProfile(request)));

  server.registerTool("evaluate_repo_plan_quality", {
    title: "Evaluate Repo Plan Quality",
    description: "Evaluate proposed stack/repo plans using hard gates, rubrics, follow-up questions, and anti reward-hacking warnings.",
    inputSchema: { request: repoQualityEvaluationInputSchema.optional() },
    outputSchema: repoQualityEvaluationOutputSchema
  }, async ({ request }) => safeJsonResponse(() => evaluateRepoPlanQuality(request)));

  server.registerTool("audit_generated_repo_quality", {
    title: "Audit Generated Repo Quality",
    description: "Audit generated repo quality after generation using hard gates and multi-dimensional rubrics.",
    inputSchema: { request: repoQualityEvaluationInputSchema.optional() },
    outputSchema: repoQualityEvaluationOutputSchema
  }, async ({ request }) => safeJsonResponse(() => auditGeneratedRepoQuality(request)));

  server.registerTool("suggest_quality_followup_questions", {
    title: "Suggest Quality Follow-up Questions",
    description: "Return focused follow-up questions when requirements, plan confidence, or quality gates are weak.",
    inputSchema: { request: repoQualityEvaluationInputSchema.optional() },
    outputSchema: qualityFollowUpOutputSchema
  }, async ({ request }) => safeJsonResponse(() => suggestQualityFollowUpQuestions(request)));

  server.registerTool("run_repo_quality_eval_scenarios", {
    title: "Run Repo Quality Eval Scenarios",
    description: "Run deterministic scenarios for repo quality hard gates and reward-hacking protections.",
    inputSchema: {},
    outputSchema: repoQualityEvalScenariosOutputSchema
  }, async () => safeJsonResponse(() => runRepoQualityEvalScenarios()));
}
