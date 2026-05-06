import { z } from "zod";

const severitySchema = z.enum(["error", "warning"]);
const stackKeySchema = z.enum(["frontend", "backend", "database", "auth", "deployment"]);
const triggerKindSchema = z.enum([
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
]);

const directoryRuleSchema = z.object({
  path: z.string().min(1),
  purpose: z.string().min(1),
  required: z.boolean()
});

const fileRuleSchema = z.object({
  name: z.string().min(1),
  rule: z.string().min(1),
  severity: severitySchema,
  trigger: z.string().min(1),
  recommendation: z.string().min(1),
  appliesToPaths: z.array(z.string()).min(1),
  goodExample: z.string().min(1).optional(),
  badExample: z.string().min(1).optional(),
  triggerKind: triggerKindSchema.optional(),
  detectors: z.array(z.object({
    kind: triggerKindSchema,
    description: z.string().min(1)
  })).optional()
});

export const stackPackSchema = z.object({
  id: z.string().regex(/^[a-z0-9-]+$/),
  name: z.string().min(1),
  version: z.string().regex(/^\d+\.\d+\.\d+$/),
  rationale: z.string().min(1),
  sources: z.array(z.object({
    label: z.string().min(1),
    url: z.string().url().optional(),
    note: z.string().min(1).optional()
  })).min(1),
  appliesTo: z.array(stackKeySchema).min(1),
  aliases: z.array(z.string().min(1)).default([]),
  directories: z.array(directoryRuleSchema),
  fileRules: z.array(fileRuleSchema),
  moduleBoundaries: z.array(z.string().min(1)),
  testingExpectations: z.array(z.string().min(1)),
  agentInstructions: z.array(z.string().min(1))
});

export type RawStackPack = z.infer<typeof stackPackSchema>;
