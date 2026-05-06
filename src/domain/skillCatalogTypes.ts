import type { StackProfile } from "./coreTypes.js";

export type SkillCatalogCategory =
  | "mcp"
  | "security"
  | "memory"
  | "github"
  | "eval"
  | "docs"
  | "frontend"
  | "database"
  | "agent_harness"
  | "verification";

export type SkillCatalogEntry = {
  id: string;
  name: string;
  category: SkillCatalogCategory;
  summary: string;
  patterns: string[];
  recommendedWhen: string[];
  cautions: string[];
  source: "built-in" | "client-supplied";
};

export type SkillCatalogQuery = {
  request?: string;
  categories?: SkillCatalogCategory[];
  stack?: StackProfile;
  suppliedSkills?: SkillCatalogEntry[];
  limit?: number;
};

export type SkillRecommendation = {
  skill: SkillCatalogEntry;
  score: number;
  confidence: "high" | "medium" | "low";
  reason: string;
};
