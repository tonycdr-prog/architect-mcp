import type { RepoArtifact } from "./types.js";

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
      continue;
    }
    for (const marker of REQUIRED_MARKERS) {
      if (!artifact.content.includes(marker)) errors.push(`${artifact.path} is missing required section or marker: ${marker}.`);
    }
  }

  return {
    valid: errors.length === 0,
    errors
  };
}
