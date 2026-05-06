import { z } from "zod";

const userLevelSchema = z.enum(["nontechnical", "beginner", "technical"]);
const repoQualityDimensionSchema = z.enum(["requirements_fit", "simplicity", "maintainability", "security", "ci_tests", "documentation", "agent_readiness", "nontechnical_suitability"]);

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

const repoQualityFindingSchema = z.object({
  code: z.string(),
  severity: z.enum(["blocker", "error", "warning"]),
  dimension: repoQualityDimensionSchema,
  message: z.string(),
  recommendation: z.string()
}).strict();

const repoQualityRubricScoreSchema = z.object({
  dimension: repoQualityDimensionSchema,
  score: z.number(),
  status: z.enum(["pass", "warn", "fail"]),
  evidence: z.array(z.string()),
  improvement: z.string()
}).strict();

export const repoQualityEvaluationOutputSchema = z.object({
  decision: z.enum(["proceed", "ask_more", "fix_before_generate", "block"]),
  stoplight: z.enum(["green", "yellow", "red"]),
  hardGates: z.array(repoQualityFindingSchema),
  scores: z.array(repoQualityRubricScoreSchema),
  overallScore: z.number(),
  confidence: z.enum(["low", "medium", "high"]),
  followUpQuestions: z.array(z.string()),
  antiRewardHackingWarnings: z.array(z.string()),
  nextActions: z.array(z.string())
}).strict();

export const qualityFollowUpOutputSchema = z.object({
  confidence: z.enum(["low", "medium", "high"]),
  questions: z.array(z.string()),
  reason: z.string()
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
