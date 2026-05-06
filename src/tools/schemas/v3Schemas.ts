import { z } from "zod";
import { stackPackCandidateSchema } from "./commonSchemas.js";

export const mcpSecurityReviewInputSchema = z.object({
  config: z.unknown(),
  approvedServers: z.array(z.string()).optional(),
  allowShell: z.boolean().optional()
}).strict();

export const v3EvalHarnessInputSchema = z.object({
  suites: z.array(z.enum(["harness", "memory", "mcp-security", "artifact-quality", "stack-pack"])).optional()
}).strict();

export const artifactQualityInputSchema = z.object({
  agentsMd: z.string().optional(),
  llmsTxt: z.string().optional()
}).strict().refine((input) => Boolean(input.agentsMd || input.llmsTxt), {
  message: "Provide agentsMd, llmsTxt, or both."
});

export const stackPackPromotionFilesSchema = z.object({
  candidate: stackPackCandidateSchema,
  writeFiles: z.boolean().optional(),
  allowOverwrite: z.boolean().optional(),
  bump: z.enum(["none", "patch", "minor", "major"]).optional()
}).strict();

export const clientRecipeInputSchema = z.object({
  recipe: z.enum(["guided-yolo-pre-edit", "repo-review-ci", "stack-pack-promotion", "memory-pr-review"]).optional()
}).strict();

export const finalResponseReviewInputSchema = z.object({
  response: z.string().min(1),
  requiredChecks: z.array(z.string()).optional()
}).strict();

export const mcpConfigFileScanInputSchema = z.object({
  rootPath: z.string().optional(),
  includeUserConfig: z.boolean().optional(),
  approvedServers: z.array(z.string()).optional()
}).strict();

export const agentSessionReviewInputSchema = z.object({
  intent: z.unknown().optional(),
  contract: z.unknown().optional(),
  changedFiles: z.array(z.unknown()).optional(),
  verification: z.array(z.object({
    check: z.string(),
    status: z.enum(["not_run", "passed", "failed", "skipped"]),
    note: z.string().optional()
  }).strict()).optional(),
  finalResponse: z.string().optional(),
  memories: z.array(z.unknown()).optional(),
  request: z.string().optional()
}).strict();

export const hostedPolicyAuditInputSchema = z.object({
  toolNames: z.array(z.string()).optional()
}).strict();
