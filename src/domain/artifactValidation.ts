import type { RepoArtifact } from "./types.js";
import { scoreAgentInstructions } from "./artifactQuality.js";

export type ArtifactValidationResult = {
  valid: boolean;
  errors: string[];
};

const REQUIRED_MARKERS = [
  "Stack Packs",
  "Pre-Coding Checklist",
  "App Generation Guardrails",
  "Agent Harness Setup",
  "Review Gate",
  "Baseline Lifecycle",
  "Architecture review"
];

export function validateRepoArtifacts(artifacts: RepoArtifact[]): ArtifactValidationResult {
  const errors: string[] = [];
  const byPath = new Map(artifacts.map((artifact) => [artifact.path, artifact]));

  for (const path of ["AGENTS.md", "docs/architecture-contract.md", ".cursor/rules/architecture.mdc", ".architectignore", "docs/build-plan.md"]) {
    if (!byPath.has(path)) errors.push(`Missing generated artifact: ${path}.`);
  }

  for (const artifact of artifacts) {
    if (!artifact.content.trim()) errors.push(`${artifact.path} is empty.`);
    if (artifact.path === ".architectignore") continue;
    if (artifact.path === "docs/build-plan.md") {
      if (!artifact.content.includes("Build Plan")) errors.push(`${artifact.path} is missing required section or marker: Build Plan.`);
      if (!/^###\s+\d+\.\s+/m.test(artifact.content) || !/Checks:\s*(npm|pnpm|yarn|bun|review_repo_structure)/i.test(artifact.content)) {
        errors.push(`${artifact.path} must include ordered slices with exact verification checks.`);
      }
      continue;
    }
    if (artifact.path === "AGENTS.md") {
      const score = scoreAgentInstructions(artifact.content);
      if (score.status === "fail") {
        errors.push(`${artifact.path} fails agent-instruction quality: ${score.findings.map((finding) => finding.message).join("; ")}`);
      }
    }
    for (const marker of REQUIRED_MARKERS) {
      if (!artifact.content.includes(marker)) errors.push(`${artifact.path} is missing required section or marker: ${marker}.`);
    }
    if (!/^##\s+/m.test(artifact.content)) errors.push(`${artifact.path} must contain real markdown sections, not marker words.`);
    if (!/npm|pnpm|yarn|bun|review_repo_structure|architecture review/i.test(artifact.content)) errors.push(`${artifact.path} must name exact verification or review commands.`);
  }

  return {
    valid: errors.length === 0,
    errors
  };
}
