import { z } from "zod";
import { fileSummarySchema, proposedFilePlanSchema, stackSchema } from "./commonSchemas.js";

export const harnessModeSchema = z.enum(["strict", "guided-yolo", "full-yolo"]);

const harnessEvidenceSchema = z.object({
  kind: z.enum(["error_output", "file_context", "repo_pattern", "user_statement", "stack_guidance"]),
  summary: z.string(),
  source: z.string().optional()
}).strict();

export const assumptionLedgerEntrySchema = z.object({
  statement: z.string(),
  reason: z.string(),
  confidence: z.enum(["high", "medium", "low"]),
  risk: z.enum(["low", "medium", "high", "critical"]),
  invalidatedBy: z.string(),
  affectedArea: z.string()
}).strict();

const triggeredStackGuidanceSchema = z.object({
  sourceId: z.string(),
  stack: z.string(),
  category: z.enum(["frontend", "backend", "database", "auth", "deployment", "payments", "ai", "orm", "platform"]),
  status: z.enum(["loaded", "metadata-only", "failed"]),
  summary: z.string(),
  evidence: z.array(z.string()),
  ruleNames: z.array(z.string()),
  warning: z.string().optional()
}).strict();

export const harnessIntentInputSchema = z.object({
  request: z.string().min(1).refine((value) => value.trim().length > 0, {
    message: "Request must not be blank."
  }),
  mode: harnessModeSchema.optional(),
  stack: stackSchema.optional(),
  selectedSourceIds: z.array(z.string()).optional(),
  files: z.array(fileSummarySchema).optional(),
  currentError: z.string().optional(),
  proposedPlan: proposedFilePlanSchema.optional(),
  verification: z.array(z.string()).optional(),
  assumptionCount: z.number().int().nonnegative().optional()
}).strict();

export const harnessIntentResultSchema = z.object({
  mode: harnessModeSchema,
  decision: z.enum(["proceed", "proceed_with_assumptions", "confirm_before_edit", "block_until_clarified"]),
  stoplight: z.enum(["green", "yellow", "red"]),
  blastRadius: z.enum(["low", "medium", "high", "critical"]),
  changeType: z.enum(["bug_fix", "refactor", "feature", "styling", "performance", "security", "data_schema", "dependency_config", "unknown"]),
  confidence: z.enum(["high", "medium", "low"]),
  interpretedProblem: z.string(),
  intendedFix: z.string(),
  plan: z.array(z.string()),
  assumptions: z.array(assumptionLedgerEntrySchema),
  nonGoals: z.array(z.string()),
  verification: z.array(z.string()),
  evidence: z.array(harnessEvidenceSchema),
  triggeredGuidance: z.array(triggeredStackGuidanceSchema),
  escalationTerms: z.array(z.string()),
  plainLanguageOptions: z.array(z.string()),
  confirmationPrompt: z.string().optional(),
  warnings: z.array(z.string()),
  handoffSummary: z.string()
}).strict();

export const preEditContractSchema = z.object({
  id: z.string(),
  createdAt: z.string(),
  mode: harnessModeSchema,
  decision: z.enum(["proceed", "proceed_with_assumptions", "confirm_before_edit", "block_until_clarified"]),
  interpretedProblem: z.string(),
  intendedBehavior: z.string(),
  changeType: z.enum(["bug_fix", "refactor", "feature", "styling", "performance", "security", "data_schema", "dependency_config", "unknown"]),
  blastRadius: z.enum(["low", "medium", "high", "critical"]),
  likelyFiles: z.array(z.string()),
  nonGoals: z.array(z.string()),
  assumptions: z.array(assumptionLedgerEntrySchema),
  evidence: z.array(harnessEvidenceSchema),
  triggeredGuidance: z.array(triggeredStackGuidanceSchema),
  verificationChecks: z.array(z.string()),
  rollbackPlan: z.string(),
  outputContract: z.array(z.string())
}).strict();

export const assumptionInputSchema = assumptionLedgerEntrySchema;

export const triggeredStackGuidanceInputSchema = z.object({
  request: z.string(),
  stack: stackSchema.optional(),
  selectedSourceIds: z.array(z.string()).optional(),
  limit: z.number().int().positive().max(12).optional()
}).strict();

export const implementationReviewSchema = z.object({
  contract: preEditContractSchema,
  changedFiles: z.array(fileSummarySchema).optional(),
  proposedPlan: proposedFilePlanSchema.optional(),
  verification: z.array(z.object({
    check: z.string(),
    status: z.enum(["not_run", "passed", "failed", "skipped"]),
    note: z.string().optional()
  }).strict()).optional()
}).strict();
