import { z } from "zod";

const userLevelSchema = z.enum(["nontechnical", "beginner", "technical"]);

export const qualityRequirementsProfileSchema = z.object({
  userLevel: userLevelSchema,
  goals: z.array(z.string()),
  constraints: z.array(z.string()),
  knownRisks: z.array(z.string()),
  missingQuestions: z.array(z.string()),
  confidence: z.enum(["low", "medium", "high"])
}).strict();

export const qualityRequirementsInputSchema = z.object({
  userLevel: userLevelSchema.optional(),
  goals: z.array(z.string()).optional(),
  constraints: z.array(z.string()).optional(),
  answers: z.array(z.string()).optional(),
  stackPreference: z.string().optional()
}).strict();

const repoQualityPlanSchema = z.object({
  stack: z.array(z.string()).optional(),
  architecture: z.string().optional(),
  tradeoffs: z.array(z.string()).optional(),
  files: z.array(z.string()).optional(),
  ciCommands: z.array(z.string()).optional(),
  testDescriptions: z.array(z.string()).optional(),
  docs: z.array(z.string()).optional(),
  envVars: z.array(z.string()).optional(),
  permissions: z.array(z.string()).optional(),
  destructiveCommands: z.array(z.string()).optional(),
  explanations: z.array(z.string()).optional()
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
