import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { realpathSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { isAbsolute, join, relative } from "node:path";
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
        summary: summarizeViolations(report.violations),
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
      const violations = reviewFileSummaries(files, contract as ArchitectureContract | undefined, maxLines, directories ?? [], buildPlan, {
        profile: reviewProfileForMode(mode)
      });
      const report = createReviewReport(violations, {
        mode,
        ignorePatterns,
        baseline: baseline as ReviewBaseline | undefined,
        maxDetailedFindings,
        summarizeLineWarningsBelow,
        gate
      });
      return {
        summary: report.summary,
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
        gate: reviewGateSchema.optional(),
        allowOutsideCwd: z.boolean().default(false)
      },
      outputSchema: reviewOutputSchema
    },
      async ({ rootPath, contract, buildPlan, directories, maxFiles, maxLines, mode, ignorePatterns, baseline, maxDetailedFindings, summarizeLineWarningsBelow, gate, allowOutsideCwd }) => safeJsonResponse(async () => {
        assertLocalScanAllowed(rootPath, { allowOutsideCwd });
        const architectIgnore = await readArchitectIgnore(rootPath);
        const combinedIgnorePatterns = [...architectIgnore, ...(ignorePatterns ?? [])];
        const scan = await scanWorkspaceWithMetadata(rootPath, maxFiles, {
          ignorePatterns: combinedIgnorePatterns
        });
        const files = scan.files;
        const violations = reviewFileSummaries(files, contract as ArchitectureContract | undefined, maxLines, directories ?? [], buildPlan, {
          profile: reviewProfileForMode(mode)
        });
        const report = createReviewReport(violations, {
          mode,
          ignorePatterns: combinedIgnorePatterns,
          baseline: baseline as ReviewBaseline | undefined,
          maxDetailedFindings,
          summarizeLineWarningsBelow,
          gate
        });
        return {
          filesReviewed: files.length,
          scan,
          summary: report.summary,
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

function assertLocalScanAllowed(rootPath: string, options: { allowOutsideCwd?: boolean }): void {
  const realRoot = realpathSync(rootPath);
  const realCwd = realpathSync(process.cwd());
  if (isInsideOrEqual(realRoot, realCwd)) return;
  if (options.allowOutsideCwd) return;
  throw new Error(`Refusing to scan ${realRoot}; review_local_workspace only scans inside ${realCwd} unless allowOutsideCwd is explicitly set for a trusted local run.`);
}

function isInsideOrEqual(child: string, parent: string): boolean {
  const rel = relative(parent, child);
  return rel === "" || (!rel.startsWith("..") && !isAbsolute(rel) && rel !== "..");
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

function reviewProfileForMode(mode: "strict" | "summary" | "ci" | "migration" | "audit"): "agent-work-gate" | "existing-repo" {
  return mode === "audit" || mode === "migration" ? "existing-repo" : "agent-work-gate";
}
