import { z } from "zod";

export const shortTextSchema = z.string().trim().min(1).max(500);
export const textSchema = z.string().trim().min(1).max(4_000);
export const documentTextSchema = z.string().trim().min(1).max(250_000);
export const textListSchema = z.array(textSchema).max(500);

export const stackSchema = z.object({
  frontend: shortTextSchema.optional(),
  backend: shortTextSchema.optional(),
  database: shortTextSchema.optional(),
  auth: shortTextSchema.optional(),
  deployment: shortTextSchema.optional()
}).strict();

export const projectBriefSchema = z.object({
  idea: textSchema,
  users: textSchema.optional(),
  coreFlows: textListSchema.optional(),
  dataEntities: textListSchema.optional(),
  stack: stackSchema.optional(),
  constraints: textListSchema.optional(),
  repoLayout: z.object({
    pathMap: z.record(shortTextSchema, z.array(shortTextSchema).max(100))
  }).strict().optional(),
  storage: textSchema.optional(),
  enforcement: textSchema.optional(),
  risk: textSchema.optional(),
  verification: textListSchema.optional()
}).strict();

export const directoryRuleSchema = z.object({
  path: z.string(),
  purpose: z.string(),
  required: z.boolean()
}).strict();

export const fileRuleSchema = z.object({
  name: z.string(),
  rule: z.string(),
  severity: z.enum(["error", "warning"]),
  trigger: z.string().optional(),
  recommendation: z.string().optional(),
  appliesToPaths: z.array(z.string()).optional(),
  goodExample: z.string().optional(),
  badExample: z.string().optional(),
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
  detectors: z.array(z.object({
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
    description: z.string()
  }).strict()).optional()
}).strict();

export const stackPackSourceSchema = z.object({
  label: z.string(),
  url: z.string().url().optional(),
  note: z.string().optional()
}).strict();

export const stackPackSchema = z.object({
  id: z.string(),
  name: z.string(),
  version: z.string(),
  rationale: z.string(),
  sources: z.array(stackPackSourceSchema),
  appliesTo: z.array(z.enum(["frontend", "backend", "database", "auth", "deployment"])),
  aliases: z.array(z.string()).optional(),
  directories: z.array(directoryRuleSchema),
  fileRules: z.array(fileRuleSchema),
  moduleBoundaries: z.array(z.string()),
  testingExpectations: z.array(z.string()),
  agentInstructions: z.array(z.string())
}).strict();

export const stackPackCandidateInputSchema = z.object({
  stackName: shortTextSchema,
  sourceText: documentTextSchema,
  sourceLabel: shortTextSchema.optional(),
  sourceUrl: z.string().url().optional()
}).strict();

export const llmsSourceQuerySchema = z.object({
  stack: z.string().optional(),
  category: z.enum(["frontend", "backend", "database", "auth", "deployment", "payments", "ai", "orm", "platform"]).optional(),
  priority: z.enum(["high", "medium", "low"]).optional()
}).strict();

export const llmsFetchSchema = z.object({
  source: z.string(),
  preferFull: z.boolean().optional(),
  maxBytes: z.number().int().positive().max(2_000_000).optional()
}).strict();

export const derivedStackPackSchema = z.object({
  sourceId: z.string()
}).strict();

export const stackPackCandidateSchema = stackPackSchema.extend({
  confidence: z.enum(["high", "medium", "low"]),
  reviewNotes: z.array(z.string())
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
  name: z.string(),
  version: z.string(),
  rationale: z.string(),
  sources: z.array(stackPackSourceSchema),
  rules: z.array(z.string()),
  artifacts: z.array(z.string()),
  reviewQuestions: z.array(z.string())
}).strict();

export const architectureContractSchema = z.object({
  contractVersion: z.string(),
  generatedBy: z.object({
    tool: z.literal("architect-mcp"),
    version: z.string(),
    generatedAt: z.string()
  }).strict(),
  name: z.string(),
  purpose: z.string(),
  stack: stackSchema,
  stackPacks: z.array(stackPackSchema),
  directories: z.array(directoryRuleSchema),
  fileRules: z.array(fileRuleSchema),
  moduleBoundaries: z.array(z.string()),
  testingExpectations: z.array(z.string()),
  agentInstructions: z.array(z.string()),
  foundationPacks: z.array(foundationPackSchema),
  archetype: appArchetypeSchema.optional()
}).strict();

export const buildPlanSchema = z.object({
  archetype: appArchetypeSchema,
  slices: z.array(z.object({
    id: z.string(),
    title: z.string(),
    order: z.number().int().positive(),
    goal: z.string(),
    inputs: z.array(z.string()),
    outputs: z.array(z.string()),
    allowedDirectories: z.array(z.string()),
    forbiddenFiles: z.array(z.string()),
    files: z.array(z.string()),
    checks: z.array(z.string()),
    stopAfter: z.string()
  }).strict()).min(1)
}).strict();

export const fileSummarySchema = z.object({
  path: z.string(),
  lines: z.number().int().nonnegative().optional(),
  bytes: z.number().int().nonnegative().optional(),
  imports: z.array(z.string()).optional(),
  hasUseClient: z.boolean().optional(),
  envAccesses: z.array(z.string()).optional(),
  hasDirectDbAccess: z.boolean().optional()
}).strict();

export const agentInstructionFormatSchema = z.enum(["agents-md", "claude-md", "cursor-rules"]);
export const reviewModeSchema = z.enum(["strict", "summary", "ci", "migration"]);

export const baselineSchema = z.object({
  findings: z.array(z.object({
    code: z.string(),
    path: z.string().optional(),
    message: z.string().optional(),
    status: z.enum(["baseline", "accepted"]).optional(),
    reason: z.string().optional()
  }).strict().refine((finding) => finding.status !== "accepted" || Boolean(finding.reason?.trim()), {
    message: "Accepted baseline findings require a reason."
  }))
}).strict();

export const reviewGateSchema = z.object({
  maxErrors: z.number().int().nonnegative().optional(),
  maxWarnings: z.number().int().nonnegative().optional(),
  minScore: z.number().min(0).max(100).optional()
}).strict();

export const proposedFilePlanSchema = z.object({
  files: z.array(z.object({
    path: z.string(),
    purpose: z.string(),
    responsibilities: z.array(z.string()).optional()
  }).strict()).min(1)
}).strict();
