import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { applyHarnessMemory, extractHarnessMemory, reviewMemoryRelevance } from "../domain/harnessMemory.js";
import type { MemoryExtractionInput, MemoryRelevanceInput } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { genericObjectOutputSchema, memoryExtractionInputSchema, memoryRelevanceInputSchema } from "./schemas.js";

export function registerMemoryTools(server: McpServer): void {
  server.registerTool(
    "extract_harness_memory",
    {
      title: "Extract Harness Memory",
      description: "Extract stateless memory proposals from harness intent, contracts, session summaries, or user statements.",
      inputSchema: {
        input: memoryExtractionInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => extractHarnessMemory(input as MemoryExtractionInput))
  );

  server.registerTool(
    "apply_harness_memory",
    {
      title: "Apply Harness Memory",
      description: "Select relevant non-red memory proposals within a token budget and disclose what was applied.",
      inputSchema: {
        input: memoryRelevanceInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => applyHarnessMemory(input as MemoryRelevanceInput))
  );

  server.registerTool(
    "review_memory_relevance",
    {
      title: "Review Memory Relevance",
      description: "Review memory proposals for relevance, sensitivity, and unsafe silent application.",
      inputSchema: {
        input: memoryRelevanceInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => reviewMemoryRelevance(input as MemoryRelevanceInput))
  );
}
