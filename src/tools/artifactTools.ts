import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { renderAgentInstructions } from "../domain/agentInstructions.js";
import { generateRepoArtifacts, generateScaffoldPlan } from "../domain/artifacts.js";
import { generateContract } from "../domain/contract.js";
import type { ProjectBrief } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { agentInstructionFormatSchema, genericObjectOutputSchema, projectBriefSchema } from "./schemas.js";

export function registerArtifactTools(server: McpServer): void {
  server.registerTool(
    "generate_agent_instructions",
    {
      title: "Generate Agent Instructions",
      description: "Export an architecture contract as AGENTS.md, CLAUDE.md, or Cursor rules.",
      inputSchema: {
        brief: projectBriefSchema,
        format: agentInstructionFormatSchema.default("agents-md"),
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ brief, format, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      return {
        format,
        text: renderAgentInstructions(contract, format)
      };
    })
  );

  server.registerTool(
    "generate_agents_md",
    {
      title: "Generate AGENTS.md",
      description: "Generate an AGENTS.md architecture instruction file from a project brief.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ brief, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      return {
        path: "AGENTS.md",
        text: renderAgentInstructions(contract, "agents-md")
      };
    })
  );

  server.registerTool(
    "generate_cursor_rules",
    {
      title: "Generate Cursor Rules",
      description: "Generate a Cursor architecture rule file from a project brief.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ brief, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      return {
        path: ".cursor/rules/architecture.mdc",
        text: renderAgentInstructions(contract, "cursor-rules")
      };
    })
  );

  server.registerTool(
    "generate_repo_artifacts",
    {
      title: "Generate Repo Artifacts",
      description: "Generate AGENTS.md, architecture contract, and Cursor rules from a project brief.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ brief, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      return {
        artifacts: generateRepoArtifacts(contract, brief as ProjectBrief)
      };
    })
  );

  server.registerTool(
    "generate_repo_scaffold_plan",
    {
      title: "Generate Repo Scaffold Plan",
      description: "Generate a directory/file scaffold plan from the selected architecture contract.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ brief, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      return {
        plan: generateScaffoldPlan(contract)
      };
    })
  );
}
