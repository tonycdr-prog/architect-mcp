import { renderAgentInstructions } from "./agentInstructions.js";
import { generateBuildPlan } from "./buildPlan.js";
import { renderContractMarkdown } from "./contract.js";
import { validateRepoArtifacts } from "./artifactValidation.js";
import {
  renderCopilotInstructions,
  renderGitHubCiWorkflow,
  renderGitHubLabelerConfig,
  renderGitHubLabelerWorkflow,
  renderPullRequestTemplate
} from "./githubRepoArtifacts.js";
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
      path: ".github/copilot-instructions.md",
      description: "GitHub Copilot repository instructions aligned with architect-mcp guardrails.",
      content: renderCopilotInstructions(contract, brief)
    },
    {
      path: ".github/labeler.yml",
      description: "GitHub labeler rules for repo areas and review ownership.",
      content: renderGitHubLabelerConfig()
    },
    {
      path: ".github/workflows/labeler.yml",
      description: "GitHub Actions workflow that applies labels to pull requests.",
      content: renderGitHubLabelerWorkflow()
    },
    {
      path: ".github/workflows/ci.yml",
      description: "GitHub Actions workflow that runs real verification checks on pull requests.",
      content: renderGitHubCiWorkflow(contract, brief)
    },
    {
      path: ".github/pull_request_template.md",
      description: "Pull request template that preserves MCP verification and handoff discipline.",
      content: renderPullRequestTemplate(contract, brief)
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
      path: ".github/copilot-instructions.md",
      action: "create-file",
      rationale: "Give GitHub Copilot repo-specific boundaries, commands, and review expectations."
    },
    {
      path: ".github/labeler.yml",
      action: "create-file",
      rationale: "Classify pull requests by repo area so review and triage do not depend on manual memory."
    },
    {
      path: ".github/workflows/labeler.yml",
      action: "create-file",
      rationale: "Run the labeler with least required permissions on pull requests."
    },
    {
      path: ".github/workflows/ci.yml",
      action: "create-file",
      rationale: "Run typecheck, tests, build, and staged MCP readiness on pull requests."
    },
    {
      path: ".github/pull_request_template.md",
      action: "create-file",
      rationale: "Require verification, MCP review, assumptions, and handoff context in every PR."
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
