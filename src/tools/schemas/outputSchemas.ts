import { z } from "zod";
import { architectureContractSchema, baselineSchema, projectBriefSchema } from "./commonSchemas.js";

export const errorOutputSchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    details: z.unknown().optional()
  }).strict()
}).strict();

export const grillMeOutputSchema = z.object({
  readinessScore: z.number(),
  specCompleteness: z.unknown(),
  ready: z.boolean(),
  phase: z.enum(["intake", "contract", "implementation"]),
  missingFields: z.array(z.string()),
  coveredFields: z.array(z.string()),
  blockers: z.array(z.string()),
  challenges: z.array(z.unknown()),
  nextQuestion: z.unknown(),
  selectedStackPacks: z.array(z.string()),
  contract: architectureContractSchema.optional(),
  markdown: z.string().optional(),
  artifacts: z.array(z.unknown()).optional(),
  scaffoldPlan: z.array(z.unknown()).optional(),
  buildPlan: z.unknown(),
  archetype: z.string(),
  foundationPacks: z.array(z.unknown()),
  updatedBrief: projectBriefSchema,
  instruction: z.string()
}).passthrough();

export const architectureContractOutputSchema = z.object({
  contract: architectureContractSchema,
  markdown: z.string()
}).strict();

export const reviewOutputSchema = z.object({
  summary: z.unknown(),
  report: z.unknown(),
  lifecycle: z.unknown().optional(),
  violations: z.array(z.unknown())
}).passthrough();

export const intakeAnswerSchema = z.object({
  field: z.enum(["idea", "users", "coreFlows", "dataEntities", "stack", "constraints", "repoLayout", "storage", "enforcement", "risk", "verification"]),
  value: z.unknown()
}).strict();

export const baselineOutputSchema = z.object({
  baseline: baselineSchema,
  summary: z.unknown()
}).strict();

export const validationOutputSchema = z.object({
  valid: z.boolean(),
  errors: z.array(z.string()),
  warnings: z.array(z.string()).optional()
}).passthrough();

export const selfReviewOutputSchema = z.object({
  grillMe: z.unknown(),
  contract: z.unknown(),
  review: z.unknown(),
  lifecycle: z.unknown()
}).strict();

export const genericObjectOutputSchema = z.object({}).passthrough();

export const readinessReportOutputSchema = z.object({
  ready: z.boolean(),
  checks: z.array(z.object({
    name: z.string(),
    status: z.enum(["pass", "warn", "fail"]),
    summary: z.string()
  }).strict())
}).strict();
