import { z } from "zod";
import { documentTextSchema, fileSummarySchema, stackPackCandidateSchema, textListSchema, textSchema } from "./commonSchemas.js";
import { harnessIntentResultSchema, preEditContractSchema } from "./harnessSchemas.js";
import { memoryProposalSchema } from "./memorySchemas.js";

export const mcpSecurityReviewInputSchema = z.object({
  config: z.unknown(),
  approvedServers: textListSchema.optional(),
  allowShell: z.boolean().optional()
}).strict();

export const v3EvalHarnessInputSchema = z.object({
  suites: z.array(z.enum(["harness", "memory", "mcp-security", "artifact-quality", "stack-pack"])).max(5).optional()
}).strict();

export const artifactQualityInputSchema = z.object({
  agentsMd: documentTextSchema.optional(),
  llmsTxt: documentTextSchema.optional()
}).strict().refine((input) => Boolean(input.agentsMd || input.llmsTxt), {
  message: "Provide agentsMd, llmsTxt, or both."
});

export const stackPackPromotionFilesSchema = z.object({
  candidate: stackPackCandidateSchema,
  writeFiles: z.boolean().optional(),
  allowOverwrite: z.boolean().optional(),
  bump: z.enum(["none", "patch", "minor", "major"]).optional(),
  packDirectory: textSchema.optional()
}).strict();

export const clientRecipeInputSchema = z.object({
  recipe: z.enum(["guided-yolo-pre-edit", "repo-review-ci", "stack-pack-promotion", "memory-pr-review"]).optional()
}).strict();

export const finalResponseReviewInputSchema = z.object({
  response: z.string().min(1),
  requiredChecks: z.array(z.string()).optional()
}).strict();

export const mcpConfigFileScanInputSchema = z.object({
  rootPath: textSchema.optional(),
  includeUserConfig: z.boolean().optional(),
  approvedServers: textListSchema.optional()
}).strict();

export const agentSessionReviewInputSchema = z.object({
  intent: harnessIntentResultSchema.optional(),
  contract: preEditContractSchema.optional(),
  changedFiles: z.array(fileSummarySchema).optional(),
  verification: z.array(z.object({
    check: textSchema,
    status: z.enum(["not_run", "passed", "failed", "skipped"]),
    note: textSchema.optional()
  }).strict()).optional(),
  finalResponse: documentTextSchema.optional(),
  memories: z.array(memoryProposalSchema).optional(),
  request: textSchema.optional()
}).strict();

export const hostedPolicyAuditInputSchema = z.object({
  toolNames: textListSchema.optional()
}).strict();
