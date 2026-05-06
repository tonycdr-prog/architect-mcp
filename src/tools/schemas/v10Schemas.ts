import { z } from "zod";

const v10AreaSchema = z.enum(["product-api", "storage", "accounts", "remote-policy", "github", "billing", "dashboard", "cli", "sdk", "mcp-loop"]);

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
