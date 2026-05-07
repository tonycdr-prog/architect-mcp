import { z } from "zod";

const userLevelSchema = z.enum(["nontechnical", "beginner", "technical"]);
const repoQualityDimensionSchema = z.enum(["requirements_fit", "simplicity", "maintainability", "security", "ci_tests", "documentation", "agent_readiness", "nontechnical_suitability"]);
const textSchema = z.string().trim().min(1).max(4_000);
const textListSchema = z.array(textSchema).max(200);

export const qualityRequirementsProfileSchema = z.object({
  userLevel: userLevelSchema,
  goals: textListSchema,
  constraints: textListSchema,
  knownRisks: textListSchema,
  missingQuestions: textListSchema,
  confidence: z.enum(["low", "medium", "high"])
}).strict();

export const qualityRequirementsInputSchema = z.object({
  userLevel: userLevelSchema.optional(),
  goals: textListSchema.optional(),
  constraints: textListSchema.optional(),
  answers: textListSchema.optional(),
  stackPreference: textSchema.optional()
}).strict();

const repoQualityPlanSchema = z.object({
  stack: textListSchema.optional(),
  architecture: textSchema.optional(),
  tradeoffs: textListSchema.optional(),
  files: textListSchema.optional(),
  ciCommands: textListSchema.optional(),
  testDescriptions: textListSchema.optional(),
  docs: textListSchema.optional(),
  envVars: textListSchema.optional(),
  permissions: textListSchema.optional(),
  destructiveCommands: textListSchema.optional(),
  explanations: textListSchema.optional()
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

const repoQualityFindingSchema = z.object({
  code: textSchema,
  severity: z.enum(["blocker", "error", "warning"]),
  dimension: repoQualityDimensionSchema,
  message: textSchema,
  recommendation: textSchema
}).strict();

const repoQualityRubricScoreSchema = z.object({
  dimension: repoQualityDimensionSchema,
  score: z.number(),
  status: z.enum(["pass", "warn", "fail"]),
  evidence: textListSchema,
  improvement: textSchema
}).strict();

export const repoQualityEvaluationOutputSchema = z.object({
  decision: z.enum(["proceed", "ask_more", "fix_before_generate", "block"]),
  stoplight: z.enum(["green", "yellow", "red"]),
  hardGates: z.array(repoQualityFindingSchema),
  scores: z.array(repoQualityRubricScoreSchema),
  overallScore: z.number(),
  confidence: z.enum(["low", "medium", "high"]),
  followUpQuestions: textListSchema,
  antiRewardHackingWarnings: textListSchema,
  nextActions: textListSchema
}).strict();

export const qualityFollowUpOutputSchema = z.object({
  confidence: z.enum(["low", "medium", "high"]),
  questions: textListSchema,
  reason: textSchema
}).strict();

export const repoQualityEvalScenariosOutputSchema = z.object({
  status: z.enum(["pass", "fail"]),
  summary: z.object({
    total: z.number(),
    passed: z.number(),
    failed: z.number()
  }).strict(),
  scenarios: z.array(z.object({
    name: z.string(),
    passed: z.boolean()
  }).strict())
}).strict();
