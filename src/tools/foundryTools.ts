import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import {
  deriveRepoConstitution,
  type RepoConstitutionArtifact,
  type RepoConstitutionPullRequest
} from "../domain/repoConstitution.js";
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

export function registerFoundryTools(server: McpServer, options: { enableLocalWorkspaceTool?: boolean } = {}): void {
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
