import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { generateContract } from "../domain/contract.js";
import { validateArchitectureContract } from "../domain/contractValidation.js";
import { diffArchitectureContracts } from "../domain/contractDiff.js";
import { validateRepoArtifacts } from "../domain/artifactValidation.js";
import { generateRepoArtifacts } from "../domain/artifacts.js";
import { runArchitectSelfReview } from "../domain/selfReview.js";
import { createMcpReadinessReport } from "../domain/readinessReport.js";
import type { ArchitectureContract, ProjectBrief } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { architectureContractSchema, readinessReportOutputSchema, selfReviewOutputSchema, validationOutputSchema, projectBriefSchema } from "./schemas.js";

export function registerValidationTools(server: McpServer): void {
  server.registerTool(
    "validate_architecture_contract",
    {
      title: "Validate Architecture Contract",
      description: "Validate that a generated architecture contract is compatible with the current architect-mcp rules and stack packs.",
      inputSchema: {
        contract: architectureContractSchema
      },
      outputSchema: validationOutputSchema
    },
    async ({ contract }) => safeJsonResponse(() => validateArchitectureContract(contract as ArchitectureContract))
  );

  server.registerTool(
    "validate_repo_artifacts",
    {
      title: "Validate Repo Artifacts",
      description: "Generate and validate repo artifacts for a brief before returning them to an agent.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: z.object({
        validation: validationOutputSchema,
        artifactPaths: z.array(z.string())
      }).strict()
    },
    async ({ brief, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      const artifacts = generateRepoArtifacts(contract);
      return {
        validation: validateRepoArtifacts(artifacts),
        artifactPaths: artifacts.map((artifact) => artifact.path)
      };
    })
  );

  server.registerTool(
    "diff_architecture_contracts",
    {
      title: "Diff Architecture Contracts",
      description: "Compare two architecture contracts and summarize breaking architecture-rule changes.",
      inputSchema: {
        before: architectureContractSchema,
        after: architectureContractSchema
      },
      outputSchema: z.object({
        breaking: z.boolean(),
        changes: z.array(z.object({
          kind: z.string(),
          severity: z.enum(["breaking", "notice"]),
          message: z.string()
        }).strict())
      }).strict()
    },
    async ({ before, after }) => safeJsonResponse(() => diffArchitectureContracts(before as ArchitectureContract, after as ArchitectureContract))
  );

  server.registerTool(
    "mcp_readiness_report",
    {
      title: "MCP Readiness Report",
      description: "Summarize architect-mcp readiness across pack validation, tool policy, self-review, hosted safety, and contract/artifact validation.",
      inputSchema: {},
      outputSchema: readinessReportOutputSchema
    },
    async () => safeJsonResponse(() => createMcpReadinessReport())
  );

  server.registerTool(
    "self_review_architect_mcp",
    {
      title: "Self Review architect-mcp",
      description: "Run /grill-me, contract generation, repo review, and lifecycle classification against the current architect-mcp workspace.",
      inputSchema: {},
      outputSchema: selfReviewOutputSchema
    },
    async () => safeJsonResponse(() => runArchitectSelfReview())
  );
}
