import { renderAgentInstructions } from "./agentInstructions.js";
import { generateBuildPlan } from "./buildPlan.js";
import { renderContractMarkdown } from "./contract.js";
import { validateRepoArtifacts } from "./artifactValidation.js";
import type { ArchitectureContract, ProjectBrief, RepoArtifact, ScaffoldPlanItem } from "./types.js";

export function generateRepoArtifacts(contract: ArchitectureContract, brief?: ProjectBrief): RepoArtifact[] {
  const artifacts = [
    {
      path: "AGENTS.md",
      description: "Repo-level instructions for coding agents.",
      content: renderAgentInstructions(contract, "agents-md")
    },
    {
      path: "docs/architecture-contract.md",
      description: "Human-readable architecture contract.",
      content: renderContractMarkdown(contract)
    },
    {
      path: ".cursor/rules/architecture.mdc",
      description: "Cursor rule file for agent standards, architecture, and verification guardrails.",
      content: renderAgentInstructions(contract, "cursor-rules")
    },
    {
      path: ".architectignore",
      description: "Architecture-review ignore patterns for generated and vendored files.",
      content: [
        "node_modules/**",
        "dist/**",
        "build/**",
        "coverage/**",
        "*.lock",
        "*.png",
        "*.jpg",
        "*.jpeg",
        "*.gif",
        "*.mp4"
      ].join("\n")
    },
    {
      path: "docs/build-plan.md",
      description: "Ordered build plan for coding agents.",
      content: renderBuildPlanMarkdown(generateBuildPlan(brief ?? {
        idea: contract.purpose,
        stack: contract.stack,
        verification: contract.testingExpectations.filter((expectation) => /^npm |^pnpm |^yarn |^bun /.test(expectation))
      }))
    }
  ];
  const validation = validateRepoArtifacts(artifacts);
  if (!validation.valid) {
    throw new Error(`Generated repo artifacts failed validation: ${validation.errors.join("; ")}`);
  }
  return artifacts;
}

export function generateScaffoldPlan(contract: ArchitectureContract): ScaffoldPlanItem[] {
  const directoryItems = contract.directories
    .filter((directory) => directory.required)
    .map<ScaffoldPlanItem>((directory) => ({
      path: directory.path,
      action: "create-directory",
      rationale: directory.purpose
    }));

  return [
    ...directoryItems,
    {
      path: "docs/architecture-contract.md",
      action: "create-file",
      rationale: "Commit the selected architecture rules before implementation begins."
    },
    {
      path: "AGENTS.md",
      action: "create-file",
      rationale: "Give future coding agents the same implementation boundaries."
    },
    {
      path: ".cursor/rules/architecture.mdc",
      action: "create-file",
      rationale: "Keep editor/agent rules in sync with the committed architecture contract."
    },
    {
      path: ".architectignore",
      action: "create-file",
      rationale: "Exclude generated, vendored, and binary files from architecture review noise."
    },
    {
      path: "docs/build-plan.md",
      action: "create-file",
      rationale: "Commit the ordered implementation slices before feature files are generated."
    },
    {
      path: "tests",
      action: "verify",
      rationale: "Make sure the repo has a test location before large generated changes land."
    },
    {
      path: "verification-commands",
      action: "verify",
      rationale: "Confirm the brief names the commands that prove generated work is acceptable."
    }
  ];
}

export function renderBuildPlanMarkdown(plan: ReturnType<typeof generateBuildPlan>): string {
  const slices = plan.slices.map((slice) => `### ${slice.order}. ${slice.title}
- ID: ${slice.id}
- Goal: ${slice.goal}
- Inputs: ${slice.inputs.join(", ")}
- Outputs: ${slice.outputs.join(", ")}
- Allowed directories: ${slice.allowedDirectories.join(", ")}
- Forbidden files: ${slice.forbiddenFiles.join(", ")}
- Planned files: ${slice.files.join(", ")}
- Checks: ${slice.checks.join(", ")}
- Stop after: ${slice.stopAfter}`).join("\n\n");

  return `# Build Plan

## App Archetype
- ${plan.archetype}

## Slices
${slices}
`;
}
