import { createFinding } from "./findingMetadata.js";
import type { BuildPlan, BuildPlanReviewOptions, ReviewViolation } from "./types.js";

export function reviewBuildPlan(plan: BuildPlan, options: BuildPlanReviewOptions = {}): ReviewViolation[] {
  const findings: ReviewViolation[] = [];
  const slices = [...plan.slices].sort((a, b) => a.order - b.order);
  const ids = slices.map((slice) => slice.id);
  const requireHarness = options.requireHarness ?? true;
  const allowedChecks = normalizeChecks(options.allowedChecks ?? checksFromContract(options.contract));

  if (requireHarness && ids[0] !== "agent-harness") {
    findings.push(createFinding({
      code: "ARCH018_BUILD_PLAN_ORDER",
      severity: "error",
      message: "Build plan does not start with the agent harness slice.",
      recommendation: "Generate AGENTS.md, docs/architecture-contract.md, and editor rules before implementation slices."
    }));
  }

  if (ids.includes("frontend-boundary") && ids.includes("database-boundary") && ids.indexOf("frontend-boundary") < ids.indexOf("database-boundary")) {
    findings.push(createFinding({
      code: "ARCH018_BUILD_PLAN_ORDER",
      severity: "error",
      message: "Frontend slice appears before the database boundary slice.",
      recommendation: "Define server-owned schema/repository boundaries before UI consumes data."
    }));
  }

  for (const slice of slices) {
    if (!slice.inputs.length || !slice.outputs.length || !slice.allowedDirectories.length) {
      findings.push(createFinding({
        code: "ARCH016_PLAN_MISSING_BOUNDARY",
        severity: "error",
        message: `Build slice ${slice.id} is missing inputs, outputs, or allowed directories.`,
        recommendation: "Each slice must define its contract before an agent writes files."
      }));
    }

    if (!slice.checks.length || slice.checks.some((check) => /^(test|typecheck|lint|verify|architecture review)$/i.test(check.trim()))) {
      findings.push(createFinding({
        code: "ARCH019_BUILD_PLAN_VERIFICATION",
        severity: "warning",
        message: `Build slice ${slice.id} has vague verification commands.`,
        recommendation: "Use exact commands such as npm run typecheck, npm test, or review_repo_structure with CI mode."
      }));
    }

    if (allowedChecks.length > 0) {
      const unknownChecks = slice.checks.filter((check) => !allowedChecks.includes(normalizeCheck(check)) && normalizeCheck(check) !== "architecture contract validates");
      if (unknownChecks.length > 0) {
        findings.push(createFinding({
          code: "ARCH019_BUILD_PLAN_VERIFICATION",
          severity: "error",
          message: `Build slice ${slice.id} uses checks outside the allowed verification set: ${unknownChecks.join(", ")}.`,
          recommendation: "Use only verification commands from the brief/contract or update the contract before accepting the plan."
        }));
      }
    }

    if (slice.forbiddenFiles.some((file) => slice.files.includes(file))) {
      findings.push(createFinding({
        code: "ARCH015_PLAN_MONOLITH_RISK",
        severity: "error",
        message: `Build slice ${slice.id} includes a forbidden monolith-prone file.`,
        recommendation: "Move implementation into feature/service/database-owned files instead of app entry files."
      }));
    }
  }

  return findings;
}

function checksFromContract(contract: BuildPlanReviewOptions["contract"]): string[] {
  return contract?.testingExpectations.filter((expectation) => /^npm |^pnpm |^yarn |^bun |^architect-mcp |^review_repo_structure/.test(expectation)) ?? [];
}

function normalizeChecks(checks: string[]): string[] {
  return checks.map(normalizeCheck);
}

function normalizeCheck(check: string): string {
  return check.trim().toLowerCase();
}
