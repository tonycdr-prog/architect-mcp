import { z } from "zod";
import { boundedArray, idText, longText, mediumText, optionalText, pathText, shortText } from "./schemaLimits.js";

export const stackSchema = z.object({
  frontend: optionalText(shortText),
  backend: optionalText(shortText),
  database: optionalText(shortText),
  auth: optionalText(shortText),
  deployment: optionalText(shortText)
}).strict();

export const projectBriefSchema = z.object({
  idea: mediumText,
  users: optionalText(mediumText),
  coreFlows: boundedArray(mediumText, 50).optional(),
  dataEntities: boundedArray(shortText, 100).optional(),
  stack: stackSchema.optional(),
  constraints: boundedArray(mediumText, 50).optional(),
  repoLayout: z.object({
    pathMap: z.record(pathText, boundedArray(pathText, 100))
  }).strict().optional(),
  storage: optionalText(mediumText),
  enforcement: optionalText(mediumText),
  risk: optionalText(mediumText),
  verification: boundedArray(mediumText, 50).optional()
}).strict();

export const directoryRuleSchema = z.object({
  path: pathText,
  purpose: mediumText,
  required: z.boolean()
}).strict();

export const fileRuleSchema = z.object({
  name: shortText,
  rule: mediumText,
  severity: z.enum(["error", "warning"]),
  trigger: optionalText(mediumText),
  recommendation: optionalText(mediumText),
  appliesToPaths: boundedArray(pathText, 100).optional(),
  goodExample: optionalText(longText),
  badExample: optionalText(longText),
  triggerKind: z.enum([
    "line-threshold",
    "import-boundary",
    "direct-db-access",
    "env-access",
    "client-boundary",
    "route-thinness",
    "migration-discipline",
    "hosted-filesystem",
    "auth-boundary",
    "payment-boundary",
    "test-policy",
    "validation-boundary",
    "ai-tool-safety",
    "manual-review"
  ]).optional(),
  detectors: boundedArray(z.object({
    kind: z.enum([
      "line-threshold",
      "import-boundary",
      "direct-db-access",
      "env-access",
      "client-boundary",
      "route-thinness",
      "migration-discipline",
      "hosted-filesystem",
      "auth-boundary",
      "payment-boundary",
      "test-policy",
      "validation-boundary",
      "ai-tool-safety",
      "manual-review"
    ]),
    description: mediumText
  }).strict(), 20).optional()
}).strict();

export const stackPackSourceSchema = z.object({
  label: shortText,
  url: z.string().url().optional(),
  note: optionalText(mediumText)
}).strict();

export const stackPackSchema = z.object({
  id: idText,
  name: shortText,
  version: shortText,
  rationale: mediumText,
  sources: boundedArray(stackPackSourceSchema, 50),
  appliesTo: boundedArray(z.enum(["frontend", "backend", "database", "auth", "deployment"]), 10),
  aliases: boundedArray(shortText, 50).optional(),
  directories: boundedArray(directoryRuleSchema, 100),
  fileRules: boundedArray(fileRuleSchema, 200),
  moduleBoundaries: boundedArray(mediumText, 100),
  testingExpectations: boundedArray(mediumText, 100),
  agentInstructions: boundedArray(mediumText, 100)
}).strict();

export const stackPackCandidateInputSchema = z.object({
  stackName: shortText,
  sourceText: longText,
  sourceLabel: optionalText(shortText),
  sourceUrl: z.string().url().optional()
}).strict();

export const llmsSourceQuerySchema = z.object({
  stack: z.string().optional(),
  category: z.enum(["frontend", "backend", "database", "auth", "deployment", "payments", "ai", "orm", "platform"]).optional(),
  priority: z.enum(["high", "medium", "low"]).optional()
}).strict();

export const llmsFetchSchema = z.object({
  source: z.string().trim().min(1).max(2_000),
  preferFull: z.boolean().optional(),
  maxBytes: z.number().int().positive().max(2_000_000).optional(),
  timeoutMs: z.number().int().positive().max(30_000).optional()
}).strict();

export const derivedStackPackSchema = z.object({
  sourceId: idText
}).strict();

export const stackPackCandidateSchema = stackPackSchema.extend({
  confidence: z.enum(["high", "medium", "low"]),
  reviewNotes: boundedArray(mediumText, 100)
}).strict();

export const appArchetypeSchema = z.enum([
  "saas-dashboard",
  "marketplace",
  "internal-admin",
  "mobile-field-app",
  "ai-workflow-tool",
  "content-docs-site",
  "custom-app"
]);

export const foundationPackSchema = z.object({
  id: z.enum(["agent-harness", "testing", "repo-structure", "ci-gates", "repo-hygiene"]),
  name: shortText,
  version: shortText,
  rationale: mediumText,
  sources: boundedArray(stackPackSourceSchema, 50),
  rules: boundedArray(mediumText, 100),
  artifacts: boundedArray(shortText, 100),
  reviewQuestions: boundedArray(mediumText, 100)
}).strict();

export const architectureContractSchema = z.object({
  contractVersion: shortText,
  generatedBy: z.object({
    tool: z.literal("architect-mcp"),
    version: shortText,
    generatedAt: shortText
  }).strict(),
  name: shortText,
  purpose: mediumText,
  stack: stackSchema,
  stackPacks: boundedArray(stackPackSchema, 50),
  directories: boundedArray(directoryRuleSchema, 200),
  fileRules: boundedArray(fileRuleSchema, 300),
  moduleBoundaries: boundedArray(mediumText, 200),
  testingExpectations: boundedArray(mediumText, 100),
  agentInstructions: boundedArray(mediumText, 100),
  foundationPacks: boundedArray(foundationPackSchema, 20),
  archetype: appArchetypeSchema.optional()
}).strict();

export const buildPlanSchema = z.object({
  archetype: appArchetypeSchema,
  slices: z.array(z.object({
    id: idText,
    title: shortText,
    order: z.number().int().positive(),
    goal: mediumText,
    inputs: boundedArray(mediumText, 50),
    outputs: boundedArray(mediumText, 50),
    allowedDirectories: boundedArray(pathText, 100),
    forbiddenFiles: boundedArray(pathText, 100),
    files: boundedArray(pathText, 200),
    checks: boundedArray(mediumText, 50),
    stopAfter: mediumText
  }).strict()).min(1).max(50)
}).strict();

export const fileSummarySchema = z.object({
  path: pathText,
  lines: z.number().int().nonnegative().optional(),
  bytes: z.number().int().nonnegative().optional(),
  imports: boundedArray(pathText, 500).optional(),
  hasUseClient: z.boolean().optional(),
  envAccesses: boundedArray(shortText, 200).optional(),
  hasDirectDbAccess: z.boolean().optional()
}).strict();

export const agentInstructionFormatSchema = z.enum(["agents-md", "claude-md", "cursor-rules"]);
export const reviewModeSchema = z.enum(["strict", "summary", "ci", "migration"]);

export const baselineSchema = z.object({
  findings: z.array(z.object({
    code: idText,
    path: pathText.optional(),
    message: optionalText(mediumText),
    status: z.enum(["baseline", "accepted"]).optional(),
    reason: optionalText(mediumText)
  }).strict().refine((finding) => finding.status !== "accepted" || Boolean(finding.reason?.trim()), {
    message: "Accepted baseline findings require a reason."
  })).max(5_000)
}).strict();

export const reviewGateSchema = z.object({
  maxErrors: z.number().int().nonnegative().optional(),
  maxWarnings: z.number().int().nonnegative().optional(),
  minScore: z.number().min(0).max(100).optional()
}).strict();

export const proposedFilePlanSchema = z.object({
  files: z.array(z.object({
    path: pathText,
    purpose: mediumText,
    responsibilities: boundedArray(mediumText, 50).optional()
  }).strict()).min(1).max(500)
}).strict();
