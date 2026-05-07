import { z } from "zod";
import { architectureContractSchema, baselineSchema, fileSummarySchema, projectBriefSchema, reviewGateSchema, reviewModeSchema, stackPackSchema } from "./commonSchemas.js";

const reviewFindingSchema = z.object({
  code: z.string(),
  confidence: z.enum(["high", "medium", "low"]),
  severity: z.enum(["error", "warning"]),
  path: z.string().optional(),
  message: z.string(),
  recommendation: z.string()
}).strict();

const reviewReportSchema = z.object({
  score: z.number().optional(),
  grade: z.string().optional(),
  mode: z.string().optional(),
  gate: z.unknown().optional(),
  summary: z.unknown().optional(),
  groups: z.array(z.unknown()).optional(),
  priorityFindings: z.array(reviewFindingSchema).optional(),
  violations: z.array(reviewFindingSchema).optional()
}).passthrough();

export const standardsProfileRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  profile: z.enum(["minimal", "balanced", "strict"]).optional(),
  stackPackIds: z.array(z.string()).optional(),
  files: z.array(fileSummarySchema).optional()
}).strict();

export const explainFindingsRequestSchema = z.object({
  findings: z.array(reviewFindingSchema).optional(),
  report: reviewReportSchema.optional(),
  context: z.string().optional()
}).strict();

export const policySimulationRequestSchema = z.object({
  findings: z.array(reviewFindingSchema).optional(),
  report: reviewReportSchema.optional(),
  baseline: baselineSchema.optional(),
  customGate: reviewGateSchema.optional()
}).strict();

export const standardsConflictRequestSchema = z.object({
  stackPacks: z.array(stackPackSchema).optional()
}).strict();

export const repoProfileFitRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  files: z.array(fileSummarySchema).optional()
}).strict();

export const contractLifecycleRequestSchema = z.object({
  before: architectureContractSchema.optional(),
  after: architectureContractSchema,
  findings: z.array(reviewFindingSchema).optional()
}).strict();

export const policyBundleRequestSchema = z.object({
  bundleId: z.string().optional(),
  brief: projectBriefSchema.optional(),
  findings: z.array(reviewFindingSchema).optional()
}).strict();

export const sessionContinuityRequestSchema = z.object({
  summaries: z.array(z.string()).optional(),
  assumptions: z.array(z.object({
    statement: z.string().optional(),
    createdAt: z.string().optional()
  }).strict()).optional(),
  maxItems: z.number().int().positive().optional()
}).strict();

export const reviewFindingsRequestSchema = z.object({
  findings: z.array(reviewFindingSchema).optional(),
  report: reviewReportSchema.optional()
}).strict();

export const localReportArtifactRequestSchema = z.object({
  title: z.string().optional(),
  report: reviewReportSchema.optional(),
  findings: z.array(reviewFindingSchema).optional(),
  format: z.enum(["markdown", "json"]).optional()
}).strict();

export const regressionCoverageRequestSchema = z.object({
  patterns: z.array(z.string()).optional(),
  fixtureNames: z.array(z.string()).optional(),
  findings: z.array(reviewFindingSchema).optional()
}).strict();

export const patternCardRequestSchema = z.object({
  card: z.object({
    id: z.string().optional(),
    summary: z.string().optional(),
    pattern: z.string().optional(),
    sensitivity: z.string().optional(),
    tokens: z.number().int().nonnegative().optional()
  }).strict(),
  brief: projectBriefSchema.optional()
}).strict();

export const strategyMapRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  stackPacks: z.array(stackPackSchema).optional()
}).strict();

export const whatIfRequestSchema = z.object({
  files: z.array(z.object({
    path: z.string(),
    purpose: z.string().optional(),
    responsibilities: z.array(z.string()).optional()
  }).strict()).optional(),
  profile: z.enum(["minimal", "balanced", "strict"]).optional(),
  findings: z.array(reviewFindingSchema).optional()
}).strict();

export const agentBehaviorRequestSchema = z.object({
  sessionSummaries: z.array(z.string()).optional(),
  reports: z.array(reviewReportSchema).optional()
}).strict();

export const ruleCandidateRequestSchema = z.object({
  sourceText: z.string().optional(),
  finding: reviewFindingSchema.optional(),
  examples: z.array(z.string()).optional()
}).strict();

export const trendSnapshotRequestSchema = z.object({
  before: reviewReportSchema.optional(),
  after: reviewReportSchema.optional()
}).strict();

export const governancePackRequestSchema = z.object({
  stackPack: stackPackSchema.optional(),
  fileRule: z.unknown().optional()
}).strict();

export const standardsRefactorRequestSchema = z.object({
  stackPacks: z.array(stackPackSchema).optional()
}).strict();

export const policyMinimizationRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  rules: z.array(z.string()).optional(),
  tokenBudget: z.number().int().positive().optional()
}).strict();

export const playbookRequestSchema = z.object({
  request: z.string().optional(),
  brief: projectBriefSchema.optional()
}).strict();

export const playbookConformanceRequestSchema = z.object({
  playbookId: z.string().optional(),
  toolsRun: z.array(z.string()).optional(),
  verification: z.array(z.string()).optional()
}).strict();

export const collaborationPlanRequestSchema = z.object({
  ownership: z.array(z.object({
    agent: z.string(),
    files: z.array(z.string())
  }).strict()).optional(),
  doNotTouch: z.array(z.string()).optional()
}).strict();

export const failureDrillRequestSchema = z.object({
  cases: z.array(z.string()).optional()
}).strict();

export const ruleImpactRequestSchema = z.object({
  findings: z.array(reviewFindingSchema).optional(),
  profile: z.enum(["minimal", "balanced", "strict"]).optional()
}).strict();

export const documentationIntelligenceRequestSchema = z.object({
  readme: z.string().optional(),
  llmsTxt: z.string().optional(),
  agentsMd: z.string().optional(),
  toolNames: z.array(z.string()).optional()
}).strict();

export const recipeRequestSchema = z.object({
  request: z.string().optional(),
  risk: z.string().optional()
}).strict();

export const scenarioAcceptanceRequestSchema = z.object({
  scenario: z.string().optional(),
  intentReady: z.boolean().optional(),
  contractReady: z.boolean().optional(),
  reviewPassed: z.boolean().optional(),
  verified: z.boolean().optional(),
  finalResponseHonest: z.boolean().optional(),
  artifactScores: z.array(z.object({ status: z.string().optional() }).passthrough()).optional()
}).strict();

export const normalizeResultRequestSchema = z.object({
  status: z.enum(["pass", "warn", "fail"]).optional(),
  findings: z.array(reviewFindingSchema).optional(),
  evidence: z.array(z.string()).optional(),
  assumptions: z.array(z.string()).optional(),
  warnings: z.array(z.string()).optional(),
  notDone: z.array(z.string()).optional(),
  handoff: z.string().optional()
}).strict();

export const contextBudgetRequestSchema = z.object({
  mode: z.enum(["compact", "standard", "full"]).optional(),
  requestedTokens: z.number().int().positive().optional(),
  findings: z.array(reviewFindingSchema).optional()
}).strict();

export const evidenceRouteRequestSchema = z.object({
  findings: z.array(reviewFindingSchema).optional(),
  sources: z.array(z.object({
    id: z.string(),
    snapshotPath: z.string().optional(),
    sha256: z.string().optional(),
    fetchedAt: z.string().optional()
  }).strict()).optional(),
  verification: z.array(z.object({
    check: z.string(),
    status: z.string()
  }).strict()).optional()
}).strict();

export const dryRunPlanRequestSchema = z.object({
  request: z.string().optional(),
  risky: z.boolean().optional(),
  expectedArtifacts: z.object({
    agentsMd: z.string().optional(),
    llmsTxt: z.string().optional()
  }).strict().optional()
}).strict();

export const toolLoopQualityRequestSchema = z.object({
  toolsRun: z.array(z.string()).optional(),
  risky: z.boolean().optional(),
  finalResponse: z.string().optional(),
  verification: z.array(z.object({ status: z.string() }).passthrough()).optional()
}).strict();

export const v5V9EvalHarnessRequestSchema = z.object({
  stage: z.enum(["v5", "v6", "v7", "v8", "v9"]).optional()
}).strict();
