import { z } from "zod";
import { architectureContractSchema, baselineSchema, fileSummarySchema, projectBriefSchema, reviewGateSchema, reviewModeSchema, stackPackSchema } from "./commonSchemas.js";
import { boundedArray, idText, longText, mediumText, optionalText, pathText, shortText } from "./schemaLimits.js";

const reviewFindingSchema = z.object({
  code: idText,
  confidence: z.enum(["high", "medium", "low"]),
  severity: z.enum(["error", "warning"]),
  path: pathText.optional(),
  message: mediumText,
  recommendation: mediumText
}).strict();

const reviewReportSchema = z.object({
  score: z.number().optional(),
  grade: z.string().optional(),
  mode: z.string().optional(),
  gate: z.unknown().optional(),
  summary: z.unknown().optional(),
  groups: boundedArray(z.unknown(), 5_000).optional(),
  priorityFindings: boundedArray(reviewFindingSchema, 5_000).optional(),
  violations: boundedArray(reviewFindingSchema, 5_000).optional()
}).passthrough();

export const standardsProfileRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  profile: z.enum(["minimal", "balanced", "strict"]).optional(),
  stackPackIds: boundedArray(idText, 100).optional(),
  files: boundedArray(fileSummarySchema, 1_000).optional()
}).strict();

export const explainFindingsRequestSchema = z.object({
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  report: reviewReportSchema.optional(),
  context: optionalText(longText)
}).strict();

export const policySimulationRequestSchema = z.object({
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  report: reviewReportSchema.optional(),
  baseline: baselineSchema.optional(),
  customGate: reviewGateSchema.optional()
}).strict();

export const standardsConflictRequestSchema = z.object({
  stackPacks: boundedArray(stackPackSchema, 50).optional()
}).strict();

export const repoProfileFitRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  files: boundedArray(fileSummarySchema, 1_000).optional()
}).strict();

export const contractLifecycleRequestSchema = z.object({
  before: architectureContractSchema.optional(),
  after: architectureContractSchema,
  findings: boundedArray(reviewFindingSchema, 5_000).optional()
}).strict();

export const policyBundleRequestSchema = z.object({
  bundleId: optionalText(idText),
  brief: projectBriefSchema.optional(),
  findings: boundedArray(reviewFindingSchema, 5_000).optional()
}).strict();

export const sessionContinuityRequestSchema = z.object({
  summaries: boundedArray(longText, 100).optional(),
  assumptions: z.array(z.object({
    statement: optionalText(mediumText),
    createdAt: optionalText(shortText)
  }).strict()).max(500).optional(),
  maxItems: z.number().int().positive().max(100).optional()
}).strict();

export const reviewFindingsRequestSchema = z.object({
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  report: reviewReportSchema.optional()
}).strict();

export const localReportArtifactRequestSchema = z.object({
  title: optionalText(shortText),
  report: reviewReportSchema.optional(),
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  format: z.enum(["markdown", "json"]).optional()
}).strict();

export const regressionCoverageRequestSchema = z.object({
  patterns: boundedArray(shortText, 500).optional(),
  fixtureNames: boundedArray(shortText, 500).optional(),
  findings: boundedArray(reviewFindingSchema, 5_000).optional()
}).strict();

export const patternCardRequestSchema = z.object({
  card: z.object({
    id: optionalText(idText),
    summary: optionalText(mediumText),
    pattern: optionalText(longText),
    sensitivity: optionalText(shortText),
    tokens: z.number().int().nonnegative().optional()
  }).strict(),
  brief: projectBriefSchema.optional()
}).strict();

export const strategyMapRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  stackPacks: boundedArray(stackPackSchema, 50).optional()
}).strict();

export const whatIfRequestSchema = z.object({
  files: z.array(z.object({
    path: pathText,
    purpose: optionalText(mediumText),
    responsibilities: boundedArray(mediumText, 50).optional()
  }).strict()).max(1_000).optional(),
  profile: z.enum(["minimal", "balanced", "strict"]).optional(),
  findings: boundedArray(reviewFindingSchema, 5_000).optional()
}).strict();

export const agentBehaviorRequestSchema = z.object({
  sessionSummaries: boundedArray(longText, 100).optional(),
  reports: boundedArray(reviewReportSchema, 50).optional()
}).strict();

export const ruleCandidateRequestSchema = z.object({
  sourceText: optionalText(longText),
  finding: reviewFindingSchema.optional(),
  examples: boundedArray(longText, 100).optional()
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
  stackPacks: boundedArray(stackPackSchema, 50).optional()
}).strict();

export const policyMinimizationRequestSchema = z.object({
  brief: projectBriefSchema.optional(),
  rules: boundedArray(mediumText, 500).optional(),
  tokenBudget: z.number().int().positive().max(200_000).optional()
}).strict();

export const playbookRequestSchema = z.object({
  request: optionalText(mediumText),
  brief: projectBriefSchema.optional()
}).strict();

export const playbookConformanceRequestSchema = z.object({
  playbookId: optionalText(idText),
  toolsRun: boundedArray(idText, 200).optional(),
  verification: boundedArray(mediumText, 100).optional()
}).strict();

export const collaborationPlanRequestSchema = z.object({
  ownership: z.array(z.object({
    agent: shortText,
    files: boundedArray(pathText, 500)
  }).strict()).max(100).optional(),
  doNotTouch: boundedArray(pathText, 500).optional()
}).strict();

export const failureDrillRequestSchema = z.object({
  cases: boundedArray(idText, 100).optional()
}).strict();

export const ruleImpactRequestSchema = z.object({
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  profile: z.enum(["minimal", "balanced", "strict"]).optional()
}).strict();

export const documentationIntelligenceRequestSchema = z.object({
  readme: optionalText(longText),
  llmsTxt: optionalText(longText),
  agentsMd: optionalText(longText),
  toolNames: boundedArray(idText, 500).optional()
}).strict();

export const recipeRequestSchema = z.object({
  request: optionalText(mediumText),
  risk: optionalText(shortText)
}).strict();

export const scenarioAcceptanceRequestSchema = z.object({
  scenario: optionalText(shortText),
  intentReady: z.boolean().optional(),
  contractReady: z.boolean().optional(),
  reviewPassed: z.boolean().optional(),
  verified: z.boolean().optional(),
  finalResponseHonest: z.boolean().optional(),
  artifactScores: z.array(z.object({ status: optionalText(shortText) }).passthrough()).max(100).optional()
}).strict();

export const normalizeResultRequestSchema = z.object({
  status: optionalText(shortText),
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  evidence: boundedArray(mediumText, 1_000).optional(),
  assumptions: boundedArray(mediumText, 1_000).optional(),
  warnings: boundedArray(mediumText, 1_000).optional(),
  notDone: boundedArray(mediumText, 1_000).optional(),
  handoff: optionalText(longText)
}).strict();

export const contextBudgetRequestSchema = z.object({
  mode: z.enum(["compact", "standard", "full"]).optional(),
  requestedTokens: z.number().int().positive().max(500_000).optional(),
  findings: boundedArray(reviewFindingSchema, 5_000).optional()
}).strict();

export const evidenceRouteRequestSchema = z.object({
  findings: boundedArray(reviewFindingSchema, 5_000).optional(),
  sources: z.array(z.object({
    id: idText,
    snapshotPath: pathText.optional(),
    sha256: optionalText(shortText),
    fetchedAt: optionalText(shortText)
  }).strict()).max(1_000).optional(),
  verification: z.array(z.object({
    check: mediumText,
    status: shortText
  }).strict()).max(200).optional()
}).strict();

export const dryRunPlanRequestSchema = z.object({
  request: optionalText(mediumText),
  risky: z.boolean().optional(),
  expectedArtifacts: z.object({
    agentsMd: optionalText(longText),
    llmsTxt: optionalText(longText)
  }).strict().optional()
}).strict();

export const toolLoopQualityRequestSchema = z.object({
  toolsRun: boundedArray(idText, 200).optional(),
  risky: z.boolean().optional(),
  finalResponse: optionalText(longText),
  verification: z.array(z.object({ status: shortText }).passthrough()).max(200).optional()
}).strict();

export const v5V9EvalHarnessRequestSchema = z.object({
  stage: z.enum(["v5", "v6", "v7", "v8", "v9"]).optional()
}).strict();
