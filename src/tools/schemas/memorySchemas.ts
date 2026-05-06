import { z } from "zod";
import { stackSchema } from "./commonSchemas.js";
import { harnessIntentResultSchema, preEditContractSchema } from "./harnessSchemas.js";

const memoryPolicySchema = z.object({
  autoStoreGreen: z.boolean().optional(),
  batchYellow: z.boolean().optional(),
  confirmRed: z.boolean().optional(),
  maxRetrievedTokens: z.number().int().positive().max(20_000).optional()
}).strict();

const memorySourceSchema = z.object({
  kind: z.enum(["harness_intent", "pre_edit_contract", "implementation_review", "user_statement", "session_summary", "manual"]),
  summary: z.string(),
  reference: z.string().optional()
}).strict();

export const memoryProposalSchema = z.object({
  id: z.string(),
  kind: z.enum(["preference", "decision", "assumption", "stack_guidance", "anti_pattern", "session_summary", "repo_fact", "accepted_risk"]),
  scope: z.enum(["user", "project", "session", "codebase"]),
  statement: z.string(),
  rationale: z.string(),
  confidence: z.enum(["high", "medium", "low"]),
  risk: z.enum(["green", "yellow", "red"]),
  sensitivity: z.enum(["public", "internal", "sensitive", "secret"]),
  policyAction: z.enum(["auto_store", "batch_review", "confirm_now", "discard"]),
  tags: z.array(z.string()),
  tokenEstimate: z.number().int().positive(),
  source: memorySourceSchema,
  invalidatedBy: z.string(),
  targetPath: z.string(),
  reviewNote: z.string().optional(),
  expiresAt: z.string().optional()
}).strict();

export const memoryExtractionInputSchema = z.object({
  request: z.string().optional(),
  intent: harnessIntentResultSchema.optional(),
  contract: preEditContractSchema.optional(),
  sessionSummary: z.string().optional(),
  projectName: z.string().optional(),
  policy: memoryPolicySchema.optional()
}).strict();

export const memoryRelevanceInputSchema = z.object({
  request: z.string().min(1),
  stack: stackSchema.optional(),
  memories: z.array(memoryProposalSchema),
  tokenBudget: z.number().int().positive().max(20_000).optional()
}).strict();
