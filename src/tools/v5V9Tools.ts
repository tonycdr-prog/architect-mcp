import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { analyzeRegressionCoverage, clusterReviewFindings, generateLocalReportArtifact, listPolicyBundles, previewPatternCard, previewPolicyBundle, reviewPatternCard, summarizeSessionContinuity, validatePolicyBundles } from "../domain/v6Governance.js";
import { compareReviewTrends, compareStandardsProfiles, createArchitectureStrategyMap, diagnoseAgentBehavior, draftRuleCandidate, previewChangeWhatIf, renderGovernancePack } from "../domain/v7Strategy.js";
import { calibrateRuleImpact, checkAgentCollaborationPlan, minimizePolicySet, reviewDocumentationIntelligence, reviewPlaybookConformance, reviewStandardsRefactor, runFailureModeDrills, selectReviewPlaybook } from "../domain/v8Automation.js";
import { createLocalDryRunPlan, evaluateScenarioAcceptance, normalizeMcpResult, planContextBudget, reviewToolLoopQuality, routeEvidence, selectLocalOrchestrationRecipe } from "../domain/v9OperatingModel.js";
import { runV5V9EvalHarness } from "../domain/v5V9EvalHarness.js";
import { analyzeStandardsConflicts, explainReviewFindings, resolveStandardsProfile, reviewContractLifecycle, scoreRepoProfileFit, simulatePolicyGate } from "../domain/v5Standards.js";
import type { ArchitectureContract, ProjectBrief, ReviewReport, ReviewViolation, StackPack } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { agentBehaviorRequestSchema, collaborationPlanRequestSchema, contextBudgetRequestSchema, contractLifecycleRequestSchema, documentationIntelligenceRequestSchema, dryRunPlanRequestSchema, evidenceRouteRequestSchema, explainFindingsRequestSchema, failureDrillRequestSchema, genericObjectOutputSchema, governancePackRequestSchema, localReportArtifactRequestSchema, normalizeResultRequestSchema, patternCardRequestSchema, playbookConformanceRequestSchema, playbookRequestSchema, policyBundleRequestSchema, policyMinimizationRequestSchema, policySimulationRequestSchema, recipeRequestSchema, regressionCoverageRequestSchema, repoProfileFitRequestSchema, reviewFindingsRequestSchema, ruleCandidateRequestSchema, ruleImpactRequestSchema, scenarioAcceptanceRequestSchema, sessionContinuityRequestSchema, standardsConflictRequestSchema, standardsProfileRequestSchema, standardsRefactorRequestSchema, strategyMapRequestSchema, toolLoopQualityRequestSchema, trendSnapshotRequestSchema, v5V9EvalHarnessRequestSchema, whatIfRequestSchema } from "./schemas.js";

export function registerV5V9Tools(server: McpServer): void {
  server.registerTool("resolve_standards_profile", {
    title: "Resolve Standards Profile",
    description: "Explain selected foundation, policy, and stack standards for minimal, balanced, or strict local review.",
    inputSchema: { request: standardsProfileRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => resolveStandardsProfile(request as Parameters<typeof resolveStandardsProfile>[0])));

  server.registerTool("explain_review_findings", {
    title: "Explain Review Findings",
    description: "Convert review findings into plain-English meaning, fix shape, next tool calls, and proof requirements.",
    inputSchema: { request: explainFindingsRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => explainReviewFindings(request as { findings?: ReviewViolation[]; report?: ReviewReport; context?: string } | undefined)));

  server.registerTool("simulate_policy_gate", {
    title: "Simulate Policy Gate",
    description: "Preview how findings behave under summary, migration, ci, strict, and custom review gates.",
    inputSchema: { request: policySimulationRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => simulatePolicyGate(request as Parameters<typeof simulatePolicyGate>[0])));

  server.registerTool("analyze_standards_conflicts", {
    title: "Analyze Standards Conflicts",
    description: "Rank duplicate or overlapping stack/foundation/policy standards and suggest resolutions.",
    inputSchema: { request: standardsConflictRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => analyzeStandardsConflicts(request as { stackPacks?: StackPack[] } | undefined)));

  server.registerTool("score_repo_profile_fit", {
    title: "Score Repo Profile Fit",
    description: "Score built-in repo profiles against supplied brief and file summaries.",
    inputSchema: { request: repoProfileFitRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => scoreRepoProfileFit(request as Parameters<typeof scoreRepoProfileFit>[0])));

  server.registerTool("review_contract_lifecycle", {
    title: "Review Contract Lifecycle",
    description: "Review contract maturity, changelog, deprecations, replacements, and noisy-rule risk.",
    inputSchema: { request: contractLifecycleRequestSchema },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => reviewContractLifecycle(request as { before?: ArchitectureContract; after: ArchitectureContract; findings?: ReviewViolation[] })));

  server.registerTool("list_policy_bundles", {
    title: "List Policy Bundles",
    description: "List local versioned policy bundles without loading hosted or remote policy state.",
    inputSchema: {},
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => listPolicyBundles()));

  server.registerTool("validate_policy_bundles", {
    title: "Validate Policy Bundles",
    description: "Validate local policy bundle ids, versions, rules, and recommendation metadata.",
    inputSchema: {},
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => validatePolicyBundles()));

  server.registerTool("preview_policy_bundle", {
    title: "Preview Policy Bundle",
    description: "Preview a local policy bundle against supplied brief and findings.",
    inputSchema: { request: policyBundleRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => previewPolicyBundle(request as { bundleId?: string; brief?: ProjectBrief; findings?: ReviewViolation[] } | undefined)));

  server.registerTool("summarize_session_continuity", {
    title: "Summarize Session Continuity",
    description: "Create stateless continuation context from explicitly supplied session summaries and assumptions.",
    inputSchema: { request: sessionContinuityRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => summarizeSessionContinuity(request)));

  server.registerTool("cluster_review_findings", {
    title: "Cluster Review Findings",
    description: "Cluster review findings by root cause, boundary, stack, or verification gap.",
    inputSchema: { request: reviewFindingsRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => clusterReviewFindings(request as { findings?: ReviewViolation[]; report?: ReviewReport } | undefined)));

  server.registerTool("generate_local_report_artifact", {
    title: "Generate Local Report Artifact",
    description: "Return Markdown and JSON report contents for clients to persist if they choose.",
    inputSchema: { request: localReportArtifactRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => generateLocalReportArtifact(request as { title?: string; report?: ReviewReport; findings?: ReviewViolation[]; format?: "markdown" | "json" } | undefined)));

  server.registerTool("analyze_regression_coverage", {
    title: "Analyze Regression Coverage",
    description: "Map known agent failure patterns to fixture coverage gaps.",
    inputSchema: { request: regressionCoverageRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => analyzeRegressionCoverage(request as { patterns?: string[]; fixtureNames?: string[]; findings?: ReviewViolation[] } | undefined)));

  server.registerTool("review_pattern_card", {
    title: "Review Pattern Card",
    description: "Validate portable local pattern cards for sensitivity, usefulness, and token size.",
    inputSchema: { request: patternCardRequestSchema },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => reviewPatternCard(request)));

  server.registerTool("preview_pattern_card", {
    title: "Preview Pattern Card",
    description: "Preview whether a local pattern card applies to a supplied brief without storing it.",
    inputSchema: { request: patternCardRequestSchema },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => previewPatternCard(request)));

  server.registerTool("create_architecture_strategy_map", {
    title: "Create Architecture Strategy Map",
    description: "Map goals, risks, stack choices, standards, boundaries, and verification.",
    inputSchema: { request: strategyMapRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => createArchitectureStrategyMap(request as { brief?: ProjectBrief; stackPacks?: StackPack[] } | undefined)));

  server.registerTool("compare_standards_profiles", {
    title: "Compare Standards Profiles",
    description: "Compare minimal, balanced, and strict standards profiles and non-negotiable safety rules.",
    inputSchema: {},
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => compareStandardsProfiles()));

  server.registerTool("preview_change_what_if", {
    title: "Preview Change What If",
    description: "Forecast review outcome, blast radius, and verification needs before edits.",
    inputSchema: { request: whatIfRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => previewChangeWhatIf(request as Parameters<typeof previewChangeWhatIf>[0])));

  server.registerTool("diagnose_agent_behavior", {
    title: "Diagnose Agent Behavior",
    description: "Identify agent failure patterns from supplied session summaries and review outputs.",
    inputSchema: { request: agentBehaviorRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => diagnoseAgentBehavior(request as { sessionSummaries?: string[]; reports?: ReviewReport[] } | undefined)));

  server.registerTool("draft_rule_candidate", {
    title: "Draft Rule Candidate",
    description: "Draft advisory local rule candidates from source text, findings, or examples.",
    inputSchema: { request: ruleCandidateRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => draftRuleCandidate(request as { sourceText?: string; finding?: ReviewViolation; examples?: string[] } | undefined)));

  server.registerTool("compare_review_trends", {
    title: "Compare Review Trends",
    description: "Compare two explicitly supplied reports for new, resolved, and changed findings.",
    inputSchema: { request: trendSnapshotRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => compareReviewTrends(request as { before?: ReviewReport; after?: ReviewReport } | undefined)));

  server.registerTool("render_governance_pack", {
    title: "Render Governance Pack",
    description: "Render a human-readable governance pack from local policy or stack-pack rules.",
    inputSchema: { request: governancePackRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => renderGovernancePack(request as Parameters<typeof renderGovernancePack>[0])));

  server.registerTool("review_standards_refactor", {
    title: "Review Standards Refactor",
    description: "Suggest splits, merges, detector families, examples, and severity changes for standards packs.",
    inputSchema: { request: standardsRefactorRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => reviewStandardsRefactor(request as { stackPacks?: StackPack[] } | undefined)));

  server.registerTool("minimize_policy_set", {
    title: "Minimize Policy Set",
    description: "Recommend keep, drop, or defer decisions for local policy rules under token pressure.",
    inputSchema: { request: policyMinimizationRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => minimizePolicySet(request)));

  server.registerTool("select_review_playbook", {
    title: "Select Review Playbook",
    description: "Select a local review playbook for feature, refactor, security, docs, or dependency work.",
    inputSchema: { request: playbookRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => selectReviewPlaybook(request)));

  server.registerTool("review_playbook_conformance", {
    title: "Review Playbook Conformance",
    description: "Check whether supplied tool calls and verification match a selected local playbook.",
    inputSchema: { request: playbookConformanceRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => reviewPlaybookConformance(request)));

  server.registerTool("check_agent_collaboration_plan", {
    title: "Check Agent Collaboration Plan",
    description: "Check file ownership, do-not-touch boundaries, and integration checklist risks.",
    inputSchema: { request: collaborationPlanRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => checkAgentCollaborationPlan(request)));

  server.registerTool("run_failure_mode_drills", {
    title: "Run Failure Mode Drills",
    description: "Run local drills for skipped verification, fake root cause, broad rewrites, leaks, and rule overreach.",
    inputSchema: { request: failureDrillRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => runFailureModeDrills(request)));

  server.registerTool("calibrate_rule_impact", {
    title: "Calibrate Rule Impact",
    description: "Preview severity, confidence, and gate-impact adjustments from supplied findings.",
    inputSchema: { request: ruleImpactRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => calibrateRuleImpact(request as { findings?: ReviewViolation[]; profile?: "minimal" | "balanced" | "strict" } | undefined)));

  server.registerTool("review_documentation_intelligence", {
    title: "Review Documentation Intelligence",
    description: "Detect stale README, llms.txt, AGENTS.md, examples, and tool-surface documentation gaps.",
    inputSchema: { request: documentationIntelligenceRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => reviewDocumentationIntelligence(request)));

  server.registerTool("select_local_orchestration_recipe", {
    title: "Select Local Orchestration Recipe",
    description: "Select deterministic local V9 recipes for common agent-work scenarios.",
    inputSchema: { request: recipeRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => selectLocalOrchestrationRecipe(request)));

  server.registerTool("evaluate_scenario_acceptance", {
    title: "Evaluate Scenario Acceptance",
    description: "Decide if a local run meets scenario-level acceptance across intent, contract, review, verification, and artifacts.",
    inputSchema: { request: scenarioAcceptanceRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => evaluateScenarioAcceptance(request)));

  server.registerTool("normalize_mcp_result", {
    title: "Normalize MCP Result",
    description: "Normalize MCP outputs into status, stoplight, findings, evidence, assumptions, proof, warnings, notDone, and handoff.",
    inputSchema: { request: normalizeResultRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => normalizeMcpResult(request as Parameters<typeof normalizeMcpResult>[0])));

  server.registerTool("plan_context_budget", {
    title: "Plan Context Budget",
    description: "Plan compact, standard, or full output budgets with evidence limits and summary precedence.",
    inputSchema: { request: contextBudgetRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => planContextBudget(request as Parameters<typeof planContextBudget>[0])));

  server.registerTool("route_evidence", {
    title: "Route Evidence",
    description: "Assign evidence ids across findings, verification checks, and source provenance.",
    inputSchema: { request: evidenceRouteRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => routeEvidence(request as Parameters<typeof routeEvidence>[0])));

  server.registerTool("create_local_dry_run_plan", {
    title: "Create Local Dry Run Plan",
    description: "Preview recipe, gates, standards, verification checks, artifacts, and final-response contract before edits.",
    inputSchema: { request: dryRunPlanRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => createLocalDryRunPlan(request)));

  server.registerTool("review_tool_loop_quality", {
    title: "Review Tool Loop Quality",
    description: "Detect skipped interpretation, missing pre-edit contracts, review without verification, and incomplete final responses.",
    inputSchema: { request: toolLoopQualityRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async ({ request }) => safeJsonResponse(() => reviewToolLoopQuality(request)));

  server.registerTool("run_v5_eval_harness", {
    title: "Run V5 Eval Harness",
    description: "Run deterministic V5 standards-intelligence evals.",
    inputSchema: { request: v5V9EvalHarnessRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => runV5V9EvalHarness("v5")));

  server.registerTool("run_v6_eval_harness", {
    title: "Run V6 Eval Harness",
    description: "Run deterministic V6 local-governance evals.",
    inputSchema: { request: v5V9EvalHarnessRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => runV5V9EvalHarness("v6")));

  server.registerTool("run_v7_eval_harness", {
    title: "Run V7 Eval Harness",
    description: "Run deterministic V7 strategic-planning evals.",
    inputSchema: { request: v5V9EvalHarnessRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => runV5V9EvalHarness("v7")));

  server.registerTool("run_v8_eval_harness", {
    title: "Run V8 Eval Harness",
    description: "Run deterministic V8 governance-automation evals.",
    inputSchema: { request: v5V9EvalHarnessRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => runV5V9EvalHarness("v8")));

  server.registerTool("run_v9_eval_harness", {
    title: "Run V9 Eval Harness",
    description: "Run deterministic V9 operating-model evals across V5-V9.",
    inputSchema: { request: v5V9EvalHarnessRequestSchema.optional() },
    outputSchema: genericObjectOutputSchema
  }, async () => safeJsonResponse(() => runV5V9EvalHarness("v9")));
}
