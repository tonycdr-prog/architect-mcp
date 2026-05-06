import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { listSkillCatalog, recommendSkills, reviewSuppliedSkills } from "../domain/skillsCatalog.js";
import type { SkillCatalogQuery } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { genericObjectOutputSchema, skillCatalogQuerySchema, suppliedSkillsReviewSchema } from "./schemas.js";

export function registerSkillCatalogTools(server: McpServer): void {
  server.registerTool(
    "list_skill_catalog",
    {
      title: "List Skill Catalog",
      description: "List built-in advisory skill patterns and optional client-supplied skill metadata.",
      inputSchema: {
        query: skillCatalogQuerySchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ query }) => safeJsonResponse(() => listSkillCatalog((query ?? {}) as SkillCatalogQuery))
  );

  server.registerTool(
    "recommend_skills_for_project",
    {
      title: "Recommend Skills For Project",
      description: "Recommend built-in or client-supplied skill patterns for a project/request without executing skill logic.",
      inputSchema: {
        query: skillCatalogQuerySchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ query }) => safeJsonResponse(() => recommendSkills(query as SkillCatalogQuery))
  );

  server.registerTool(
    "review_supplied_skills",
    {
      title: "Review Supplied Skills",
      description: "Review external skill metadata before using it as advisory source material.",
      inputSchema: {
        input: suppliedSkillsReviewSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => reviewSuppliedSkills(input))
  );
}
