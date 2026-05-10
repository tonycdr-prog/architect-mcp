import { z } from "zod";
import { boundedArray, idText, mediumText, optionalText, pathText, shortText } from "./schemaLimits.js";

export const mcpServerCategorySchema = z.enum(["database", "payments", "workspace", "deployment", "testing", "memory", "developer-tools"]);
export const mcpServerTrustTierSchema = z.enum(["vendor-official", "reference", "community-vetted", "unknown"]);
export const mcpInstallTargetClientSchema = z.enum(["codex", "claude-code", "cursor", "vscode", "generic-json"]);

export const mcpCatalogQuerySchema = z.object({
  category: mcpServerCategorySchema.optional(),
  provider: optionalText(shortText),
  trustTier: mcpServerTrustTierSchema.optional(),
  query: optionalText(shortText)
}).strict();

export const mcpRecommendationInputSchema = z.object({
  request: optionalText(mediumText),
  projectBrief: z.object({
    idea: optionalText(mediumText),
    users: optionalText(mediumText),
    coreFlows: boundedArray(mediumText, 50).optional(),
    stack: z.record(shortText, optionalText(shortText)).optional(),
    constraints: boundedArray(mediumText, 50).optional()
  }).strict().optional(),
  selectedStackPacks: boundedArray(shortText, 50).optional(),
  confirmedProviders: boundedArray(shortText, 20).optional(),
  serverSideBoundaryConfirmed: z.boolean().optional()
}).strict();

export const mcpInstallPlanRequestSchema = z.object({
  serverId: idText,
  targetClient: mcpInstallTargetClientSchema.optional(),
  hostedMode: z.boolean().optional()
}).strict();

const mcpGeneratedPlanSchema = z.object({
  id: shortText,
  serverId: idText,
  serverName: shortText,
  targetClient: mcpInstallTargetClientSchema,
  status: z.literal("dry-run"),
  hostedMode: z.boolean(),
  localOnly: z.boolean(),
  requiresApproval: z.literal(true),
  writeFiles: z.literal(false),
  packagePin: shortText,
  mcpConfig: z.object({
    mcpServers: z.record(shortText, z.record(shortText, z.unknown()))
  }).strict(),
  clientConfig: z.record(shortText, z.unknown()),
  env: boundedArray(shortText, 50),
  postInstall: boundedArray(mediumText, 50),
  warnings: boundedArray(mediumText, 50)
}).strict();

export const mcpInstallPlanReviewSchema = z.object({
  plan: mcpGeneratedPlanSchema,
  writeFiles: z.boolean().optional(),
  explicitApproval: z.boolean().optional()
}).strict();

export const mcpInstallPlanApplySchema = z.object({
  plan: mcpGeneratedPlanSchema,
  targetPath: pathText.optional(),
  writeFiles: z.boolean().optional(),
  explicitApproval: z.boolean().optional()
}).strict();
