import { z } from "zod";
import { stackSchema } from "./commonSchemas.js";

const skillCatalogCategorySchema = z.enum(["mcp", "security", "memory", "github", "eval", "docs", "frontend", "database", "agent_harness", "verification"]);

export const skillCatalogEntrySchema = z.object({
  id: z.string(),
  name: z.string(),
  category: skillCatalogCategorySchema,
  summary: z.string(),
  patterns: z.array(z.string()),
  recommendedWhen: z.array(z.string()),
  cautions: z.array(z.string()),
  source: z.enum(["built-in", "client-supplied"])
}).strict();

export const skillCatalogQuerySchema = z.object({
  request: z.string().optional(),
  categories: z.array(skillCatalogCategorySchema).optional(),
  stack: stackSchema.optional(),
  suppliedSkills: z.array(skillCatalogEntrySchema).optional(),
  limit: z.number().int().positive().max(50).optional()
}).strict();

export const suppliedSkillsReviewSchema = z.object({
  skills: z.array(skillCatalogEntrySchema)
}).strict();
