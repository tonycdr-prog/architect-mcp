import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { inferRepoLayoutFromFiles } from "../domain/repoLayout.js";
import { safeJsonResponse } from "./responses.js";
import { genericObjectOutputSchema } from "./schemas.js";

export function registerLayoutTools(server: McpServer): void {
  server.registerTool(
    "infer_repo_layout",
    {
      title: "Infer Repo Layout",
      description: "Infer a repo layout mapping from file paths so stack packs can adapt to existing repos.",
      inputSchema: {
        paths: z.array(z.string()).min(1)
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ paths }) => safeJsonResponse(() => ({
      repoLayout: inferRepoLayoutFromFiles(paths)
    }))
  );
}
