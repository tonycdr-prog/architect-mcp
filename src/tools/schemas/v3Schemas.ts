import { z } from "zod";
import { fileSummarySchema, stackPackCandidateSchema } from "./commonSchemas.js";
import { harnessIntentResultSchema, preEditContractSchema } from "./harnessSchemas.js";
import { memoryProposalSchema } from "./memorySchemas.js";
import { boundedArray, idText, longText, mediumText, optionalText, pathText } from "./schemaLimits.js";
import { untrustedInputSources } from "../../domain/untrustedInputs.js";
import { verificationReceiptSources } from "../../domain/verificationReceipts.js";
import { workGateSequence } from "../../domain/workGateCompleteness.js";

const untrustedInputSchema = z.object({
  source: z.enum(untrustedInputSources)
}).strict();

const workGateNameSchema = z.enum(workGateSequence);
const verificationReceiptSchema = z.object({
  command: mediumText,
  status: z.enum(["not_run", "passed", "failed", "skipped"]),
  source: z.enum(verificationReceiptSources),
  summary: mediumText,
  recordedAt: optionalText(mediumText),
  runId: optionalText(idText)
}).strict();

export const workGateCompletenessInputSchema = z.object({
  records: boundedArray(z.object({
    gate: workGateNameSchema,
    status: z.enum(["pass", "warn", "fail", "unknown"]).optional(),
    recordedAt: optionalText(mediumText),
    runId: optionalText(idText)
  }).strict(), 200).optional(),
  requiredGates: boundedArray(workGateNameSchema, workGateSequence.length).optional(),
  now: optionalText(mediumText),
  maxAgeSeconds: z.number().int().positive().max(31_536_000).optional()
}).strict();

export const mcpSecurityReviewInputSchema = z.object({
  config: z.unknown(),
  approvedServers: boundedArray(idText, 200).optional(),
  allowShell: z.boolean().optional()
}).strict();

export const v3EvalHarnessInputSchema = z.object({
  suites: z.array(z.enum(["harness", "memory", "mcp-security", "artifact-quality", "stack-pack"])).max(5).optional()
}).strict();

export const artifactQualityInputSchema = z.object({
  agentsMd: optionalText(longText),
  llmsTxt: optionalText(longText)
}).strict().refine((input) => Boolean(input.agentsMd || input.llmsTxt), {
  message: "Provide agentsMd, llmsTxt, or both."
});

export const stackPackPromotionFilesSchema = z.object({
  candidate: stackPackCandidateSchema,
  writeFiles: z.boolean().optional(),
  allowOverwrite: z.boolean().optional(),
  bump: z.enum(["none", "patch", "minor", "major"]).optional(),
  packDirectory: optionalText(pathText)
}).strict();

export const clientRecipeInputSchema = z.object({
  recipe: z.enum(["guided-yolo-pre-edit", "repo-review-ci", "stack-pack-promotion", "memory-pr-review"]).optional()
}).strict();

export const finalResponseReviewInputSchema = z.object({
  response: longText,
  requiredChecks: boundedArray(mediumText, 100).optional(),
  untrustedInputs: boundedArray(untrustedInputSchema, 100).optional(),
  verificationReceipts: boundedArray(verificationReceiptSchema, 100).optional(),
  receiptNow: optionalText(mediumText),
  receiptMaxAgeSeconds: z.number().int().positive().max(31_536_000).optional()
}).strict();

export const mcpConfigFileScanInputSchema = z.object({
  rootPath: pathText.optional(),
  includeUserConfig: z.boolean().optional(),
  approvedServers: boundedArray(idText, 200).optional()
}).strict();

export const agentSessionReviewInputSchema = z.object({
  intent: harnessIntentResultSchema.optional(),
  contract: preEditContractSchema.optional(),
  changedFiles: boundedArray(fileSummarySchema, 1_000).optional(),
  verification: z.array(z.object({
    check: mediumText,
    status: z.enum(["not_run", "passed", "failed", "skipped"]),
    note: optionalText(mediumText)
  }).strict()).max(100).optional(),
  verificationReceipts: boundedArray(verificationReceiptSchema, 100).optional(),
  receiptNow: optionalText(mediumText),
  receiptMaxAgeSeconds: z.number().int().positive().max(31_536_000).optional(),
  finalResponse: optionalText(longText),
  memories: boundedArray(memoryProposalSchema, 200).optional(),
  request: optionalText(mediumText),
  untrustedInputs: boundedArray(untrustedInputSchema, 100).optional()
}).strict();

export const hostedPolicyAuditInputSchema = z.object({
  toolNames: boundedArray(idText, 500).optional()
}).strict();
