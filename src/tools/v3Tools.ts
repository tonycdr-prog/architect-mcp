import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { reviewAgentSession } from "../domain/agentSessionReview.js";
import { scoreAgentInstructions, scoreLlmsTxt } from "../domain/artifactQuality.js";
import { listClientIntegrationRecipes } from "../domain/clientRecipes.js";
import { reviewAgentFinalResponse } from "../domain/finalResponseReview.js";
import { scanMcpConfigFiles } from "../domain/mcpConfigScanner.js";
import { reviewMcpConfigSecurity } from "../domain/mcpSecurity.js";
import { createStackPackCoverageMatrix, scoreStackPacks } from "../domain/packReports.js";
import { promoteStackPackCandidateToFiles } from "../domain/stackPackWorkflow.js";
import { classifyToolPolicy } from "../domain/toolPolicy.js";
import type { StackPackCandidate } from "../domain/types.js";
import { runV3EvalHarness } from "../domain/v3EvalHarness.js";
import { safeJsonResponse } from "./responses.js";
import { agentSessionReviewInputSchema, artifactQualityInputSchema, clientRecipeInputSchema, finalResponseReviewInputSchema, genericObjectOutputSchema, hostedPolicyAuditInputSchema, mcpConfigFileScanInputSchema, mcpSecurityReviewInputSchema, stackPackPromotionFilesSchema, v3EvalHarnessInputSchema } from "./schemas.js";

export function registerV3Tools(server: McpServer, options: { enableLocalWorkspaceTool?: boolean } = {}): void {
  server.registerTool(
    "review_mcp_config_security",
    {
      title: "Review MCP Config Security",
      description: "Audit MCP server config objects for secrets, shell injection, unpinned dependencies, and unapproved servers.",
      inputSchema: {
        request: mcpSecurityReviewInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => reviewMcpConfigSecurity(request))
  );

  server.registerTool(
    "run_v3_eval_harness",
    {
      title: "Run V3 Eval Harness",
      description: "Run deterministic V3 behavior evals for harness, memory, MCP security, artifact quality, and stack-pack workflows.",
      inputSchema: {
        request: v3EvalHarnessInputSchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => runV3EvalHarness(request ?? {}))
  );

  server.registerTool(
    "score_agent_artifacts",
    {
      title: "Score Agent Artifacts",
      description: "Score generated AGENTS.md and llms.txt content for operational quality and LLM navigation usefulness.",
      inputSchema: {
        request: artifactQualityInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => ({
      agentsMd: request.agentsMd ? scoreAgentInstructions(request.agentsMd) : undefined,
      llmsTxt: request.llmsTxt ? scoreLlmsTxt(request.llmsTxt) : undefined
    }))
  );

  server.registerTool(
    "promote_stack_pack_to_files",
    {
      title: "Promote Stack Pack To Files",
      description: "Validate a reviewed stack-pack candidate and return or write versioned packs/<id>.json plus packs/manifest.json updates.",
      inputSchema: {
        request: stackPackPromotionFilesSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => promoteStackPackCandidateToFiles(request.candidate as StackPackCandidate, {
      writeFiles: request.writeFiles,
      allowOverwrite: request.allowOverwrite,
      bump: request.bump
    }))
  );

  server.registerTool(
    "list_client_integration_recipes",
    {
      title: "List Client Integration Recipes",
      description: "Return executable client-side call recipes for harness gates, CI review, stack-pack promotion, and memory review.",
      inputSchema: {
        request: clientRecipeInputSchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => listClientIntegrationRecipes(request ?? {}))
  );

  server.registerTool(
    "review_agent_final_response",
    {
      title: "Review Agent Final Response",
      description: "Check final agent responses for changed, verified, assumptions, not-done, and evidence honesty.",
      inputSchema: {
        request: finalResponseReviewInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => reviewAgentFinalResponse(request))
  );

  server.registerTool(
    "review_agent_session",
    {
      title: "Review Agent Session",
      description: "Combine intent, pre-edit contract, implementation drift, final response, memory, and verification honesty into one harness report.",
      inputSchema: {
        request: agentSessionReviewInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => reviewAgentSession({
      intent: request.intent,
      contract: request.contract,
      changedFiles: request.changedFiles,
      verification: request.verification,
      finalResponse: request.finalResponse,
      memories: request.memories,
      request: request.request
    }))
  );

  server.registerTool(
    "audit_hosted_tool_policy",
    {
      title: "Audit Hosted Tool Policy",
      description: "Classify tools as hosted-safe, local-only, or future-adapter for hosted deployment planning.",
      inputSchema: {
        request: hostedPolicyAuditInputSchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => {
      const knownToolNames = registeredToolNames(options.enableLocalWorkspaceTool !== false);
      return safeJsonResponse(() => classifyToolPolicy(request?.toolNames ?? knownToolNames, { knownToolNames }));
    }
  );

  server.registerTool(
    "score_stack_packs",
    {
      title: "Score Stack Packs",
      description: "Score stack packs for source quality, detector coverage, examples, tests, and boundaries.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => scoreStackPacks())
  );

  server.registerTool(
    "stack_pack_coverage_matrix",
    {
      title: "Stack Pack Coverage Matrix",
      description: "Report which stack packs have ingested llms.txt snapshots, detectors, tests, and docs.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => createStackPackCoverageMatrix())
  );

  if (options.enableLocalWorkspaceTool !== false) {
    server.registerTool(
      "scan_mcp_config_files",
      {
        title: "Scan MCP Config Files",
        description: "Local-only scan of repo and optional user MCP config files before running MCP security review.",
        inputSchema: {
          request: mcpConfigFileScanInputSchema.optional()
        },
        outputSchema: genericObjectOutputSchema
      },
      async ({ request }) => safeJsonResponse(() => scanMcpConfigFiles(request ?? {}))
    );
  }
}

function registeredToolNames(includeLocal: boolean): string[] {
  return [
    "list_stack_packs", "validate_stack_packs", "list_foundation_packs", "validate_foundation_packs",
    "stack_pack_expansion_strategy", "discover_llms_sources", "fetch_llms_source", "ingest_llms_txt",
    "list_ingested_llms_sources", "derive_stack_pack_from_llms_source", "propose_stack_pack_rules",
    "review_stack_pack_candidate", "promote_stack_pack_candidate", "promote_stack_pack_to_files",
    "analyze_stack_pack_conflicts", "diff_stack_pack_versions", "list_repo_profiles", "get_repo_profile",
    "infer_repo_layout", "start_project_intake", "grill_project_brief", "grill_me", "continue_grill_me",
    "generate_architecture_contract", "generate_agent_instructions", "generate_agents_md", "generate_cursor_rules",
    "generate_repo_artifacts", "generate_repo_scaffold_plan", "review_build_plan", "review_proposed_file_plan",
    "review_repo_structure", "create_review_baseline", "validate_architecture_contract", "validate_repo_artifacts",
    "self_review_architect_mcp", "diff_architecture_contracts", "mcp_readiness_report",
    "interpret_implementation_intent", "classify_ambiguity_risk", "create_pre_edit_contract",
    "review_implementation_against_contract", "record_assumption", "load_triggered_stack_guidance",
    "extract_harness_memory", "apply_harness_memory", "review_memory_relevance", "list_skill_catalog",
    "recommend_skills_for_project", "review_supplied_skills", "review_mcp_config_security",
    "run_v3_eval_harness", "score_agent_artifacts", "list_client_integration_recipes",
    "review_agent_final_response", "review_agent_session", "audit_hosted_tool_policy", "score_stack_packs",
    "stack_pack_coverage_matrix",
    "resolve_standards_profile", "explain_review_findings", "simulate_policy_gate",
    "analyze_standards_conflicts", "score_repo_profile_fit", "review_contract_lifecycle",
    "run_v5_eval_harness", "list_policy_bundles", "validate_policy_bundles", "preview_policy_bundle",
    "summarize_session_continuity", "cluster_review_findings", "generate_local_report_artifact",
    "analyze_regression_coverage", "review_pattern_card", "preview_pattern_card", "run_v6_eval_harness",
    "create_architecture_strategy_map", "compare_standards_profiles", "preview_change_what_if",
    "diagnose_agent_behavior", "draft_rule_candidate", "compare_review_trends", "render_governance_pack",
    "run_v7_eval_harness", "review_standards_refactor", "minimize_policy_set", "select_review_playbook",
    "review_playbook_conformance", "check_agent_collaboration_plan", "run_failure_mode_drills",
    "calibrate_rule_impact", "review_documentation_intelligence", "run_v8_eval_harness",
    "select_local_orchestration_recipe", "evaluate_scenario_acceptance", "normalize_mcp_result",
    "plan_context_budget", "route_evidence", "create_local_dry_run_plan", "review_tool_loop_quality",
    "run_v9_eval_harness",
    ...(includeLocal ? ["review_local_workspace", "scan_mcp_config_files"] : [])
  ];
}
