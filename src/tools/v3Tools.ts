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
import { auditWorkGateCompleteness } from "../domain/workGateCompleteness.js";
import { createWorkGateSequenceReceipt } from "../domain/workGateSequenceReceipt.js";
import { safeJsonResponse } from "./responses.js";
import { agentSessionReviewInputSchema, artifactQualityInputSchema, clientRecipeInputSchema, finalResponseReviewInputSchema, genericObjectOutputSchema, hostedPolicyAuditInputSchema, mcpConfigFileScanInputSchema, mcpSecurityReviewInputSchema, stackPackPromotionFilesSchema, v3EvalHarnessInputSchema, workGateCompletenessInputSchema, workGateSequenceReceiptInputSchema } from "./schemas.js";
import { registeredArchitectureToolNames, type ToolSurface } from "./toolRegistry.js";

export function registerV3Tools(server: McpServer, options: { enableLocalWorkspaceTool?: boolean; toolSurface?: ToolSurface } = {}): void {
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
      bump: request.bump,
      packDirectory: request.packDirectory
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
      request: request.request,
      untrustedInputs: request.untrustedInputs
    }))
  );

  server.registerTool(
    "audit_work_gate_completeness",
    {
      title: "Audit Work Gate Completeness",
      description: "Read-only report on whether direct MCP clients supplied complete, fresh, ordered work-gate evidence.",
      inputSchema: {
        request: workGateCompletenessInputSchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => auditWorkGateCompleteness(request ?? {}))
  );

  server.registerTool(
    "create_work_gate_sequence_receipt",
    {
      title: "Create Work Gate Sequence Receipt",
      description: "Create a public-safe direct-client receipt for ordered work-gate calls, confirmed inputs, and review evidence.",
      inputSchema: {
        request: workGateSequenceReceiptInputSchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => createWorkGateSequenceReceipt(request ?? {}))
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
      const knownToolNames = registeredArchitectureToolNames(options.enableLocalWorkspaceTool !== false, options.toolSurface ?? "core");
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
