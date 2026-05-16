import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  applyMcpInstallPlan,
  createMcpInstallPlan,
  listMcpServerCatalog,
  recommendMcpServers,
  reviewMcpInstallPlan
} from "../domain/mcpCatalog.js";
import type {
  McpCatalogQuery,
  McpInstallPlan,
  McpRecommendationInput
} from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import {
  genericObjectOutputSchema,
  mcpCatalogQuerySchema,
  mcpInstallPlanApplySchema,
  mcpInstallPlanRequestSchema,
  mcpInstallPlanReviewSchema,
  mcpRecommendationInputSchema
} from "./schemas.js";

export function registerMcpCatalogTools(server: McpServer): void {
  server.registerTool(
    "list_mcp_server_catalog",
    {
      title: "List MCP Server Catalog",
      description: "List the curated, source-backed MCP server catalog used for guarded recommendations.",
      inputSchema: {
        query: mcpCatalogQuerySchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ query }) => safeJsonResponse(() => listMcpServerCatalog((query ?? {}) as McpCatalogQuery))
  );

  server.registerTool(
    "recommend_mcp_servers",
    {
      title: "Recommend MCP Servers",
      description: "Recommend MCP servers from the local catalog after provider and boundary checks; asks clarifying questions instead of guessing.",
      inputSchema: {
        request: mcpRecommendationInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => recommendMcpServers(request as McpRecommendationInput))
  );

  server.registerTool(
    "create_mcp_install_plan",
    {
      title: "Create MCP Install Plan",
      description: "Create a dry-run MCP client config plan from a catalog entry without writing files.",
      inputSchema: {
        request: mcpInstallPlanRequestSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => createMcpInstallPlan(request))
  );

  server.registerTool(
    "review_mcp_install_plan",
    {
      title: "Review MCP Install Plan",
      description: "Review an MCP install plan for unknown servers, secret leakage, unpinned packages, and unsafe writes.",
      inputSchema: {
        request: mcpInstallPlanReviewSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => reviewMcpInstallPlan(request as { plan: McpInstallPlan; writeFiles?: boolean; explicitApproval?: boolean }))
  );

  server.registerTool(
    "apply_mcp_install_plan",
    {
      title: "Apply MCP Install Plan",
      description: "Local-only dry-run/apply tool for merging a reviewed MCP install plan into a project-local JSON config.",
      inputSchema: {
        request: mcpInstallPlanApplySchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => applyMcpInstallPlan(request as { plan: McpInstallPlan; targetPath?: string; writeFiles?: boolean; explicitApproval?: boolean }))
  );
}
