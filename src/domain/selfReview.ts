import { createBaselineFromFindings } from "./baseline.js";
import { generateContract } from "./contract.js";
import { grillMe } from "./intake.js";
import { createReviewReport } from "./reviewReport.js";
import { reviewFileSummaries } from "./reviewer.js";
import { classifyReviewLifecycle } from "./reviewLifecycle.js";
import type { ProjectBrief } from "./types.js";
import { scanWorkspace } from "../infrastructure/scanWorkspace.js";

export const ARCHITECT_MCP_BRIEF: ProjectBrief = {
  idea: "Local-first MCP standards and verification harness for coding agents. It grills project briefs, generates architecture contracts, creates repo artifacts, reviews implementation drift, and enforces evidence before completion.",
  users: "Builders using coding agents who need maintainable implementation standards before and after code changes.",
  coreFlows: ["grill project brief", "generate architecture contract", "review repo structure", "review agent session", "classify baseline lifecycle"],
  stack: {
    backend: "TypeScript MCP server",
    deployment: "Node HTTP Streamable MCP"
  },
  storage: "No persistence until hosted review history, teams, or billing is needed.",
  enforcement: "Local advisory review with explicit pass/warn/fail gates; CI adapter later.",
  repoLayout: {
    pathMap: {
      "src/domain": ["src/domain"],
      "src/tools": ["src/tools"],
      "src/server": ["src/server"],
      "src/infrastructure": ["src/infrastructure"],
      "tests": ["tests"]
    }
  },
  risk: "Agents creating monolithic files, mixing MCP tool wiring into domain logic, exposing hosted filesystem scans, or accepting vague best-practice packs.",
  verification: ["npm run typecheck", "npm test", "npm run build", "npm run check:v10"]
};

export async function runArchitectSelfReview(rootPath = process.cwd()) {
  const grilled = grillMe(ARCHITECT_MCP_BRIEF, {
    stackPackIds: ["mcp-server"],
    includeContract: true,
    includeArtifacts: true
  });
  const contract = grilled.contract ?? generateContract(ARCHITECT_MCP_BRIEF, ["mcp-server"]);
  const files = await scanWorkspace(rootPath, 5000);
  const directories = inferDirectories(files.map((file) => file.path));
  const findings = reviewFileSummaries(files, contract, 300, directories);
  const report = createReviewReport(findings, {
    mode: "ci",
    gate: {
      maxWarnings: 7
    }
  });
  const baseline = createBaselineFromFindings(findings);

  return {
    grillMe: {
      ready: grilled.ready,
      readinessScore: grilled.readinessScore,
      blockers: grilled.blockers,
      challenges: grilled.challenges,
      selectedStackPacks: grilled.selectedStackPacks
    },
    contract: {
      contractVersion: contract.contractVersion,
      generatedBy: contract.generatedBy,
      stackPacks: contract.stackPacks.map((pack) => ({
        id: pack.id,
        version: pack.version
      }))
    },
    review: {
      filesReviewed: files.length,
      findings: findings.length,
      summary: report.summary,
      gate: report.gate,
      grade: report.grade,
      score: report.score,
      priorityFindings: report.priorityFindings
    },
    lifecycle: classifyReviewLifecycle(findings, baseline)
  };
}

function inferDirectories(paths: string[]): string[] {
  return [...new Set(paths.flatMap((path) => {
    const parts = path.split("/");
    return parts.slice(0, -1).map((_, index) => parts.slice(0, index + 1).join("/"));
  }))];
}
