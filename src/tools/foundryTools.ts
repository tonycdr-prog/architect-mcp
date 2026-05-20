import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import {
  deriveRepoConstitution,
  type RepoConstitutionArtifact,
  type RepoConstitutionPullRequest
} from "../domain/repoConstitution.js";
import { normalizeFoundryEvidence, type FoundryEvidenceInventoryInput } from "../domain/foundryEvidence.js";
import { deriveLocalRepoConstitution } from "../infrastructure/repoConstitutionWorkspace.js";
import { safeJsonResponse } from "./responses.js";
import { fileSummarySchema, genericObjectOutputSchema } from "./schemas.js";

const constitutionArtifactSchema = z.object({
  path: z.string(),
  content: z.string()
}).strict();

const recentPullRequestSchema = z.object({
  number: z.number().int().positive().optional(),
  title: z.string().optional(),
  author: z.string().optional(),
  authorAssociation: z.string().optional(),
  merged: z.boolean().optional(),
  body: z.string().optional()
}).strict();

const reviewFindingSchema = z.object({
  code: z.string(),
  confidence: z.enum(["high", "medium", "low"]),
  severity: z.enum(["error", "warning"]),
  path: z.string().optional(),
  message: z.string(),
  recommendation: z.string()
}).strict();

const reviewReportSchema = z.object({
  priorityFindings: z.array(reviewFindingSchema).optional(),
  violations: z.array(reviewFindingSchema).optional(),
  coverage: z.object({
    scanTruncated: z.boolean().optional(),
    detailedFindingsTruncated: z.boolean().optional(),
    filesReviewed: z.number().int().nonnegative().optional(),
    maxFiles: z.number().int().nonnegative().optional(),
    topScannedDirectories: z.array(z.object({
      directory: z.string(),
      files: z.number().int().nonnegative()
    }).strict()).optional(),
    findingHistogram: z.array(z.object({
      code: z.string(),
      severity: z.string(),
      count: z.number().int().nonnegative()
    }).strict()).optional(),
    caveats: z.array(z.string()).optional()
  }).passthrough().optional()
}).passthrough();

const externalFindingSchema = z.object({
  toolName: z.string(),
  ruleId: z.string().optional(),
  severity: z.enum(["error", "warning", "info"]).optional(),
  confidence: z.enum(["high", "medium", "low"]).optional(),
  path: z.string().optional(),
  message: z.string(),
  recommendation: z.string().optional(),
  rawPayload: z.unknown().optional(),
  securitySensitive: z.boolean().optional()
}).strict();

const verificationEvidenceSchema = z.object({
  check: z.string(),
  status: z.enum(["passed", "failed", "skipped", "not_run", "unknown"]),
  summary: z.string().optional(),
  recordedAt: z.string().optional()
}).strict();

const repoConstitutionSchema = z.object({
  schemaVersion: z.literal(1),
  findings: z.array(z.object({
    code: z.string(),
    severity: z.enum(["info", "warning"]),
    message: z.string(),
    recommendation: z.string()
  }).strict()),
  pullRequests: z.object({
    templates: z.array(z.object({
      path: z.string(),
      contentProvided: z.boolean(),
      headings: z.array(z.string()),
      checklistItems: z.number()
    }).passthrough())
  }).passthrough()
}).passthrough();

export function registerFoundryTools(server: McpServer, options: { enableLocalWorkspaceTool?: boolean } = {}): void {
  server.registerTool(
    "normalize_foundry_evidence",
    {
      title: "Normalize Foundry Evidence",
      description: "Normalize review findings, external tool findings, verification, repo constitution, coverage, and suppression candidates into a public-safe evidence inventory.",
      inputSchema: {
        findings: z.array(reviewFindingSchema).optional(),
        reviewReports: z.array(reviewReportSchema).optional(),
        externalFindings: z.array(externalFindingSchema).optional(),
        verification: z.array(verificationEvidenceSchema).optional(),
        repoConstitution: repoConstitutionSchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ findings, reviewReports, externalFindings, verification, repoConstitution }) => safeJsonResponse(() => ({
      inventory: normalizeFoundryEvidence({
        findings: findings as FoundryEvidenceInventoryInput["findings"],
        reviewReports: reviewReports as FoundryEvidenceInventoryInput["reviewReports"],
        externalFindings,
        verification,
        repoConstitution
      })
    }))
  );

  server.registerTool(
    "derive_repo_constitution",
    {
      title: "Derive Repo Constitution",
      description: "Derive repo instructions, templates, CI, release, package, and advisory recent-PR style signals from supplied public-safe repo evidence.",
      inputSchema: {
        files: z.array(fileSummarySchema).optional(),
        artifacts: z.array(constitutionArtifactSchema).optional(),
        recentPullRequests: z.array(recentPullRequestSchema).optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ files, artifacts, recentPullRequests }) => safeJsonResponse(() => ({
      constitution: deriveRepoConstitution({
        files,
        artifacts: artifacts as RepoConstitutionArtifact[] | undefined,
        recentPullRequests: recentPullRequests as RepoConstitutionPullRequest[] | undefined
      })
    }))
  );

  if (options.enableLocalWorkspaceTool ?? true) {
    server.registerTool(
      "derive_local_repo_constitution",
      {
        title: "Derive Local Repo Constitution",
        description: "Scan a trusted local workspace and derive public-safe repo constitution signals without mutating files.",
        inputSchema: {
          rootPath: z.string(),
          maxFiles: z.number().int().positive().max(5000).default(1000),
          allowOutsideCwd: z.boolean().default(false),
          recentPullRequests: z.array(recentPullRequestSchema).optional()
        },
        outputSchema: genericObjectOutputSchema
      },
      async ({ rootPath, maxFiles, allowOutsideCwd, recentPullRequests }) => safeJsonResponse(async () => {
        return deriveLocalRepoConstitution({
          rootPath,
          maxFiles,
          allowOutsideCwd,
          recentPullRequests: recentPullRequests as RepoConstitutionPullRequest[] | undefined
        });
      })
    );
  }
}
