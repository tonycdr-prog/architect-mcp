import { z } from "zod";
import { boundedArray, mediumText, optionalText, shortText } from "./schemaLimits.js";

const userLevelSchema = z.enum(["nontechnical", "beginner", "technical"]);

export const qualityRequirementsProfileSchema = z.object({
  userLevel: userLevelSchema,
  goals: boundedArray(mediumText, 50),
  constraints: boundedArray(mediumText, 50),
  knownRisks: boundedArray(shortText, 50),
  missingQuestions: boundedArray(mediumText, 20),
  confidence: z.enum(["low", "medium", "high"])
}).strict();

export const qualityRequirementsInputSchema = z.object({
  userLevel: userLevelSchema.optional(),
  goals: boundedArray(mediumText, 50).optional(),
  constraints: boundedArray(mediumText, 50).optional(),
  answers: boundedArray(mediumText, 100).optional(),
  stackPreference: optionalText(shortText)
}).strict();

const repoQualityPlanSchema = z.object({
  stack: boundedArray(shortText, 50).optional(),
  architecture: optionalText(mediumText),
  tradeoffs: boundedArray(mediumText, 50).optional(),
  files: boundedArray(shortText, 500).optional(),
  ciCommands: boundedArray(mediumText, 100).optional(),
  testDescriptions: boundedArray(mediumText, 100).optional(),
  docs: boundedArray(mediumText, 100).optional(),
  envVars: boundedArray(shortText, 100).optional(),
  permissions: boundedArray(mediumText, 100).optional(),
  destructiveCommands: boundedArray(mediumText, 50).optional(),
  explanations: boundedArray(mediumText, 100).optional()
}).strict();

const repoQualitySignalsSchema = z.object({
  hasReadme: z.boolean().optional(),
  hasSetupInstructions: z.boolean().optional(),
  hasEnvExample: z.boolean().optional(),
  hasAgentsMd: z.boolean().optional(),
  agentsMdVague: z.boolean().optional(),
  hasMeaningfulCi: z.boolean().optional(),
  ciOnlyEchoes: z.boolean().optional(),
  hasMeaningfulTests: z.boolean().optional(),
  testsAreTrivial: z.boolean().optional(),
  hasHardcodedSecrets: z.boolean().optional(),
  unsafePermissions: z.boolean().optional(),
  overcomplicatedStack: z.boolean().optional(),
  underpoweredStack: z.boolean().optional(),
  jargonHeavy: z.boolean().optional(),
  explainsTradeoffs: z.boolean().optional()
}).strict();

export const repoQualityEvaluationInputSchema = z.object({
  profile: qualityRequirementsProfileSchema.optional(),
  plan: repoQualityPlanSchema.optional(),
  signals: repoQualitySignalsSchema.optional()
}).strict();
