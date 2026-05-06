import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { generateContract, renderContractMarkdown } from "../domain/contract.js";
import type { ProjectBrief } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { architectureContractOutputSchema, projectBriefSchema } from "./schemas.js";

export function registerContractTools(server: McpServer): void {
  server.registerTool(
    "generate_architecture_contract",
    {
      title: "Generate Architecture Contract",
      description: "Generate a repo architecture contract that coding agents can implement against.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional()
      },
      outputSchema: architectureContractOutputSchema
    },
    async ({ brief, stackPackIds }) => safeJsonResponse(() => {
      const contract = generateContract(brief as ProjectBrief, stackPackIds ?? []);
      return {
        contract,
        markdown: renderContractMarkdown(contract)
      };
    })
  );
}
