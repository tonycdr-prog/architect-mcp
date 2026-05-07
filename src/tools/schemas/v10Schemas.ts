import { z } from "zod";

const v10AreaSchema = z.enum(["product-api", "storage", "accounts", "remote-policy", "github", "billing", "dashboard", "cli", "sdk", "mcp-loop"]);
const v10StatusSchema = z.enum(["pass", "warn", "fail"]);

export const v10BlueprintRequestSchema = z.object({
  area: z.union([v10AreaSchema, z.literal("all")]).optional()
}).strict();

export const v10SlicePlanRequestSchema = z.object({
  sliceId: z.string().optional()
}).strict();

export const v10DashboardPlanRequestSchema = z.object({
  screenRoute: z.string().optional()
}).strict();

export const v10BoundaryReviewRequestSchema = z.object({
  routes: z.array(z.object({
    path: z.string(),
    tenantScoped: z.boolean().optional(),
    repositoryBoundary: z.string().optional(),
    auth: z.string().optional()
  }).strict()).optional(),
  storageEntities: z.array(z.object({
    table: z.string(),
    orgScoped: z.boolean().optional(),
    purpose: z.string().optional()
  }).strict()).optional(),
  dashboardScreens: z.array(z.object({
    route: z.string(),
    primerComponents: z.array(z.string()).optional(),
    dataSources: z.array(z.string()).optional()
  }).strict()).optional(),
  policyRollouts: z.array(z.object({
    mode: z.string().optional(),
    preservesLocalInstructions: z.boolean().optional(),
    hasEmergencyDisable: z.boolean().optional()
  }).strict()).optional(),
  billingGates: z.array(z.object({
    feature: z.string(),
    gatesLocalMcp: z.boolean().optional()
  }).strict()).optional()
}).strict();

const v10RouteSchema = z.object({
  method: z.enum(["GET", "POST", "PATCH", "DELETE"]),
  path: z.string(),
  area: v10AreaSchema,
  auth: z.string(),
  tenantScoped: z.boolean(),
  repositoryBoundary: z.string(),
  notes: z.array(z.string()),
  idempotency: z.boolean()
}).strict();

const v10StorageEntitySchema = z.object({
  table: z.string(),
  area: v10AreaSchema,
  orgScoped: z.boolean(),
  purpose: z.string(),
  sensitiveColumns: z.array(z.string()),
  retention: z.string()
}).strict();

const v10DashboardScreenSchema = z.object({
  route: z.string(),
  title: z.string(),
  primerComponents: z.array(z.string()),
  dataSources: z.array(z.string()),
  accessibilityChecks: z.array(z.string())
}).strict();

const v10FindingSchema = z.object({
  code: z.string(),
  status: v10StatusSchema,
  area: v10AreaSchema,
  message: z.string(),
  recommendation: z.string()
}).strict();

const v10ImplementationSliceSchema = z.object({
  id: z.string(),
  title: z.string(),
  areas: z.array(v10AreaSchema),
  likelyFiles: z.array(z.string()),
  verification: z.array(z.string()),
  mcpToolsBefore: z.array(z.string()),
  mcpToolsAfter: z.array(z.string()),
  stopCondition: z.string()
}).strict();

export const v10BlueprintOutputSchema = z.object({
  version: z.literal("v10"),
  status: z.string(),
  principle: z.string(),
  routes: z.array(v10RouteSchema),
  storage: z.array(v10StorageEntitySchema),
  repositoryBoundaries: z.array(z.object({
    name: z.string(),
    owns: z.array(z.string()),
    forbiddenConsumers: z.array(z.string()),
    requiredChecks: z.array(z.string())
  }).strict()),
  dashboardScreens: z.array(v10DashboardScreenSchema),
  implementationSlices: z.array(v10ImplementationSliceSchema),
  nonNegotiables: z.array(z.string())
}).strict();

export const v10SlicePlanOutputSchema = z.object({
  slices: z.array(v10ImplementationSliceSchema),
  mcpLoopRequired: z.boolean(),
  beforeEverySlice: z.array(z.string()),
  afterEverySlice: z.array(z.string()),
  stopRule: z.string()
}).strict();

export const v10DashboardPlanOutputSchema = z.object({
  primerSource: z.string(),
  constraints: z.array(z.string()),
  screens: z.array(v10DashboardScreenSchema),
  sharedLayout: z.record(z.string(), z.array(z.string()))
}).strict();

export const v10BoundaryReviewOutputSchema = z.object({
  status: v10StatusSchema,
  findings: z.array(v10FindingSchema),
  summary: z.object({
    failures: z.number(),
    warnings: z.number()
  }).strict()
}).strict();

export const v10EvalHarnessOutputSchema = z.object({
  stage: z.literal("v10"),
  status: z.enum(["pass", "fail"]),
  summary: z.object({
    total: z.number(),
    passed: z.number(),
    failed: z.number()
  }).strict(),
  results: z.array(z.object({
    name: z.string(),
    passed: z.boolean()
  }).strict())
}).strict();
