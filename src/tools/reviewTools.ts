import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { createBaselineFromFindings } from "../domain/baseline.js";
import { reviewBuildPlan } from "../domain/buildPlanReviewer.js";
import { createReviewReport } from "../domain/reviewReport.js";
import { reviewFileSummaries } from "../domain/reviewer.js";
import { reviewProposedFilePlan } from "../domain/planReviewer.js";
import { classifyReviewLifecycle } from "../domain/reviewLifecycle.js";
import type { ArchitectureContract, ReviewBaseline } from "../domain/types.js";
import { scanWorkspaceWithMetadata } from "../infrastructure/scanWorkspace.js";
import { safeJsonResponse, summarizeViolations } from "./responses.js";
import {
  architectureContractSchema,
  baselineOutputSchema,
  baselineSchema,
  buildPlanSchema,
  fileSummarySchema,
  proposedFilePlanSchema,
  reviewGateSchema,
  reviewModeSchema,
  reviewOutputSchema
} from "./schemas.js";

export type ReviewToolsOptions = {
  enableLocalWorkspaceTool?: boolean;
};

export function registerReviewTools(server: McpServer, options: ReviewToolsOptions = {}): void {
  server.registerTool(
    "review_build_plan",
    {
      title: "Review Build Plan",
      description: "Review an ordered build plan before implementation starts.",
      inputSchema: {
        plan: buildPlanSchema,
        contract: architectureContractSchema.optional(),
        allowedChecks: z.array(z.string()).optional()
      },
      outputSchema: reviewOutputSchema
    },
    async ({ plan, contract, allowedChecks }) => safeJsonResponse(() => {
      const violations = reviewBuildPlan(plan, {
        contract: contract as ArchitectureContract | undefined,
        allowedChecks
      });
      const report = createReviewReport(violations, { mode: "ci" });
      return {
        summary: summarizeViolations(violations),
        report,
        violations: report.violations
      };
    })
  );

  server.registerTool(
    "review_proposed_file_plan",
    {
      title: "Review Proposed File Plan",
      description: "Review a proposed scaffold/build file plan before an agent writes files.",
      inputSchema: {
        plan: proposedFilePlanSchema
      },
      outputSchema: reviewOutputSchema
    },
    async ({ plan }) => safeJsonResponse(() => {
      const violations = reviewProposedFilePlan(plan);
      const report = createReviewReport(violations, { mode: "ci" });
      return {
        summary: summarizeViolations(violations),
        report,
        violations: report.violations
      };
    })
  );

  server.registerTool(
    "review_repo_structure",
    {
      title: "Review Repo Structure",
      description: "Review a file tree for architecture, repo hygiene, stack-boundary, and implementation-drift violations.",
      inputSchema: {
        files: z.array(fileSummarySchema),
        directories: z.array(z.string()).optional(),
        contract: architectureContractSchema.optional(),
        buildPlan: buildPlanSchema.optional(),
        maxLines: z.number().int().positive().default(300),
        mode: reviewModeSchema.default("summary"),
        ignorePatterns: z.array(z.string()).optional(),
        baseline: baselineSchema.optional(),
        maxDetailedFindings: z.number().int().positive().optional(),
        summarizeLineWarningsBelow: z.number().int().nonnegative().optional(),
        gate: reviewGateSchema.optional()
      },
      outputSchema: reviewOutputSchema
    },
    async ({ files, directories, contract, buildPlan, maxLines, mode, ignorePatterns, baseline, maxDetailedFindings, summarizeLineWarningsBelow, gate }) => safeJsonResponse(() => {
      const violations = reviewFileSummaries(files, contract as ArchitectureContract | undefined, maxLines, directories ?? [], buildPlan);
      const report = createReviewReport(violations, {
        mode,
        ignorePatterns,
        baseline: baseline as ReviewBaseline | undefined,
        maxDetailedFindings,
        summarizeLineWarningsBelow,
        gate
      });
      return {
        summary: summarizeViolations(violations),
        report,
        lifecycle: baseline ? classifyReviewLifecycle(violations, baseline as ReviewBaseline) : undefined,
        violations: report.violations
      };
    })
  );

  if (options.enableLocalWorkspaceTool ?? true) {
    server.registerTool(
      "review_local_workspace",
      {
        title: "Review Local Workspace",
        description: "Scan a local workspace and review it for architecture, repo hygiene, stack-boundary, and implementation-drift violations.",
        inputSchema: {
        rootPath: z.string(),
        contract: architectureContractSchema.optional(),
        buildPlan: buildPlanSchema.optional(),
        directories: z.array(z.string()).optional(),
        maxFiles: z.number().int().positive().max(5000).default(1000),
        maxLines: z.number().int().positive().default(300),
        mode: reviewModeSchema.default("summary"),
        ignorePatterns: z.array(z.string()).optional(),
        baseline: baselineSchema.optional(),
        maxDetailedFindings: z.number().int().positive().optional(),
        summarizeLineWarningsBelow: z.number().int().nonnegative().optional(),
        gate: reviewGateSchema.optional()
      },
      outputSchema: reviewOutputSchema
    },
      async ({ rootPath, contract, buildPlan, directories, maxFiles, maxLines, mode, ignorePatterns, baseline, maxDetailedFindings, summarizeLineWarningsBelow, gate }) => safeJsonResponse(async () => {
        const architectIgnore = await readArchitectIgnore(rootPath);
        const scan = await scanWorkspaceWithMetadata(rootPath, maxFiles, [...architectIgnore, ...(ignorePatterns ?? [])]);
        const files = scan.files;
        const violations = reviewFileSummaries(files, contract as ArchitectureContract | undefined, maxLines, directories ?? [], buildPlan);
        const report = createReviewReport(violations, {
          mode,
          ignorePatterns: [...architectIgnore, ...(ignorePatterns ?? [])],
          baseline: baseline as ReviewBaseline | undefined,
          maxDetailedFindings,
          summarizeLineWarningsBelow,
          gate
        });
        return {
          filesReviewed: files.length,
          scan: {
            filesReviewed: files.length,
            truncated: scan.truncated,
            maxFiles: scan.maxFiles
          },
          warnings: scan.truncated ? [`Workspace scan reached maxFiles=${scan.maxFiles}; review may be incomplete.`] : [],
          summary: summarizeViolations(violations),
          report,
          lifecycle: baseline ? classifyReviewLifecycle(violations, baseline as ReviewBaseline) : undefined,
          violations: report.violations
        };
      })
    );
  }

  server.registerTool(
    "create_review_baseline",
    {
      title: "Create Review Baseline",
      description: "Create a baseline from current architecture findings so CI can fail only on new findings later.",
      inputSchema: {
        files: z.array(fileSummarySchema),
        directories: z.array(z.string()).optional(),
        contract: architectureContractSchema.optional(),
        buildPlan: buildPlanSchema.optional(),
        maxLines: z.number().int().positive().default(300),
        ignorePatterns: z.array(z.string()).optional()
      },
      outputSchema: baselineOutputSchema
    },
    async ({ files, directories, contract, buildPlan, maxLines, ignorePatterns }) => safeJsonResponse(() => {
      const violations = reviewFileSummaries(files, contract as ArchitectureContract | undefined, maxLines, directories ?? [], buildPlan);
      const report = createReviewReport(violations, {
        mode: "strict",
        ignorePatterns
      });

      return {
        baseline: createBaselineFromFindings(report.violations),
        summary: report.summary
      };
    })
  );
}

async function readArchitectIgnore(rootPath: string): Promise<string[]> {
  try {
    const content = await readFile(join(rootPath, ".architectignore"), "utf8");
    return content
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("#"));
  } catch {
    return [];
  }
}
