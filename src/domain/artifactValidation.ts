import type { RepoArtifact } from "./types.js";
import { isConcreteVerificationCommand } from "./verificationCommands.js";

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

  for (const path of [
    "AGENTS.md",
    "docs/architecture-contract.md",
    ".cursor/rules/architecture.mdc",
    ".architectignore",
    ".github/copilot-instructions.md",
    ".github/labeler.yml",
    ".github/workflows/labeler.yml",
    ".github/workflows/ci.yml",
    ".github/pull_request_template.md",
    "docs/build-plan.md"
  ]) {
    if (!byPath.has(path)) errors.push(`Missing generated artifact: ${path}.`);
  }

  for (const artifact of artifacts) {
    if (!artifact.content.trim()) errors.push(`${artifact.path} is empty.`);
    if (artifact.path === ".architectignore") continue;
    if (artifact.path === ".github/labeler.yml") {
      if (!/changed-files:/i.test(artifact.content)) errors.push(`${artifact.path} must define changed-files label rules.`);
      if (!/^domain:/m.test(artifact.content) || !/^tools:/m.test(artifact.content) || !/^tests:/m.test(artifact.content)) {
        errors.push(`${artifact.path} must use valid YAML label keys with colons.`);
      }
      if (!/src\/domain\/\*\*/.test(artifact.content) || !/src\/tools\/\*\*/.test(artifact.content) || !/tests\/\*\*/.test(artifact.content)) {
        errors.push(`${artifact.path} must label domain, tools, and tests changes.`);
      }
      continue;
    }
    if (artifact.path === ".github/workflows/labeler.yml") {
      if (!/actions\/labeler@v\d+/i.test(artifact.content)) errors.push(`${artifact.path} must run actions/labeler with a pinned major version.`);
      if (!/pull-requests:\s*write/i.test(artifact.content) || !/contents:\s*read/i.test(artifact.content)) {
        errors.push(`${artifact.path} must use least required labeler permissions.`);
      }
      if (/pull_request_target:/i.test(artifact.content) && /actions\/checkout@/i.test(artifact.content)) {
        errors.push(`${artifact.path} must not checkout pull request code when using pull_request_target.`);
      }
      continue;
    }
    if (artifact.path === ".github/workflows/ci.yml") {
      if (!/pull_request:/i.test(artifact.content)) errors.push(`${artifact.path} must run on pull requests.`);
      if (!/contents:\s*read/i.test(artifact.content)) errors.push(`${artifact.path} must use read-only repository permissions.`);
      if (!/actions\/checkout@v\d+/i.test(artifact.content)) {
        errors.push(`${artifact.path} must checkout code with a pinned major action.`);
      }
      const runCommands = extractRunCommands(artifact.content);
      const concreteCommands = runCommands.filter(isConcreteVerificationCommand);
      if (concreteCommands.length === 0 && !/No verification command was specified/i.test(artifact.content)) {
        errors.push(`${artifact.path} must run at least one concrete verification command or fail closed when none is known.`);
      }
      continue;
    }
    if (artifact.path === ".github/pull_request_template.md") {
      if (!/Verification/i.test(artifact.content) || !/MCP Review/i.test(artifact.content) || !/Handoff/i.test(artifact.content)) {
        errors.push(`${artifact.path} must require verification, MCP review, and handoff sections.`);
      }
      continue;
    }
    if (artifact.path === ".github/copilot-instructions.md") {
      if (!extractCodeSpans(artifact.content).some(isConcreteVerificationCommand) && !/No verification command was provided/i.test(artifact.content)) {
        errors.push(`${artifact.path} must include concrete verification commands or say they are missing.`);
      }
      if (!/src\/domain|src\/tools|src\/server/i.test(artifact.content)) {
        errors.push(`${artifact.path} must describe repo architecture boundaries.`);
      }
      if (!/evidence|root cause|complete/i.test(artifact.content)) {
        errors.push(`${artifact.path} must require evidence before completion claims.`);
      }
      continue;
    }
    if (artifact.path === "docs/build-plan.md") {
      if (!artifact.content.includes("Build Plan")) errors.push(`${artifact.path} is missing required section or marker: Build Plan.`);
      if (!/^###\s+\d+\.\s+/m.test(artifact.content)) errors.push(`${artifact.path} must define ordered implementation slices.`);
      if (!/- Checks:\s+\S+/m.test(artifact.content)) errors.push(`${artifact.path} must name verification checks for each slice.`);
      if (!/- Stop after:\s+\S+/m.test(artifact.content)) errors.push(`${artifact.path} must state stop conditions for agent handoff.`);
      continue;
    }
    for (const marker of REQUIRED_MARKERS) {
      if (!artifact.content.includes(marker)) errors.push(`${artifact.path} is missing required section or marker: ${marker}.`);
    }

    if (artifact.path === "AGENTS.md") {
      if (!extractVerificationCommandCandidates(artifact.content).some(isConcreteVerificationCommand)) {
        errors.push(`${artifact.path} must include concrete verification commands.`);
      }
      if (!/Do Not Create/i.test(artifact.content) || !/(App\.tsx|page\.tsx|server\.ts|route\.ts|index\.ts)/i.test(artifact.content)) {
        errors.push(`${artifact.path} must name forbidden monolith files.`);
      }
      if (!/proof|verified|verification|evidence/i.test(artifact.content)) {
        errors.push(`${artifact.path} must require evidence before completion claims.`);
      }
    }

    if (artifact.path === "docs/architecture-contract.md" || artifact.path === ".cursor/rules/architecture.mdc") {
      if (!/mode|threshold|max errors|max warnings|fail/i.test(artifact.content)) {
        errors.push(`${artifact.path} must describe review gate mode or thresholds.`);
      }
      if (!/Stack Packs/i.test(artifact.content) || !/Foundation Packs/i.test(artifact.content)) {
        errors.push(`${artifact.path} must name selected stack and foundation packs.`);
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors
  };
}

function extractRunCommands(content: string): string[] {
  return [...content.matchAll(/^\s*-\s+run:\s+(.+)$/gm)].map((match) => unquoteYamlScalar(match[1].trim()));
}

function unquoteYamlScalar(value: string): string {
  if ((value.startsWith("\"") && value.endsWith("\"")) || (value.startsWith("'") && value.endsWith("'"))) {
    try {
      return JSON.parse(value);
    } catch {
      return value.slice(1, -1);
    }
  }
  return value;
}

function extractCodeSpans(content: string): string[] {
  return [...content.matchAll(/`([^`]+)`/g)].map((match) => match[1].trim());
}

function extractVerificationCommandCandidates(content: string): string[] {
  const listItems = content
    .split("\n")
    .map((line) => line.trim().replace(/^[-*]\s+/, "").trim());
  return [...extractCodeSpans(content), ...listItems];
}
