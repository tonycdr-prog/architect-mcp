import { z } from "zod";
import { stackSchema } from "./commonSchemas.js";
import { boundedArray, idText, mediumText, optionalText, shortText } from "./schemaLimits.js";

const skillCatalogCategorySchema = z.enum(["mcp", "security", "memory", "github", "eval", "docs", "frontend", "database", "agent_harness", "verification"]);

export const skillCatalogEntrySchema = z.object({
  id: idText,
  name: shortText,
  category: skillCatalogCategorySchema,
  summary: mediumText,
  patterns: boundedArray(shortText, 50),
  recommendedWhen: boundedArray(shortText, 50),
  cautions: boundedArray(mediumText, 50),
  source: z.enum(["built-in", "client-supplied"])
}).strict();

export const skillCatalogQuerySchema = z.object({
  request: optionalText(mediumText),
  categories: boundedArray(skillCatalogCategorySchema, 20).optional(),
  stack: stackSchema.optional(),
  suppliedSkills: boundedArray(skillCatalogEntrySchema, 100).optional(),
  limit: z.number().int().positive().max(50).optional()
}).strict();

export const suppliedSkillsReviewSchema = z.object({
  skills: boundedArray(skillCatalogEntrySchema, 100)
}).strict();
