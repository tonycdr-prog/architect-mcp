import type { ArchitectureContract, ProjectBrief } from "./types.js";
import { extractVerificationCommands, packageManagerForVerificationCommands } from "./verificationCommands.js";

export function renderCopilotInstructions(contract: ArchitectureContract, brief?: ProjectBrief): string {
  const commands = verificationCommandsFor(contract, brief);
  const commandLines = commands.length
    ? commands.map((command) => `- \`${command}\``).join("\n")
    : "- No verification command was provided. Ask the user to define one before relying on generated CI.";

  return `# GitHub Copilot Instructions

This repository implements: ${contract.purpose}

## Commands
${commandLines}

## Architecture Boundaries
- Domain decisions live in \`src/domain/*\`.
- MCP registration, schemas, and response formatting live in \`src/tools/*\`.
- Server construction and transport boundaries live in \`src/server/*\`, \`src/http.ts\`, and \`src/index.ts\`.
- Filesystem and host adapters live in \`src/infrastructure/*\`.
- Tests live in \`tests/*\`.

## Review Rules
- Do not create giant aggregation files or monolithic route/tool modules.
- Do not add hosted, database, GitHub, billing, account, or dashboard runtime behavior unless the current task explicitly targets that layer.
- Keep local MCP safety behavior available without paid, hosted, or remote services.
- Require evidence before claiming a root cause or marking work complete.
- Update generated artifact tests when changing repo artifact generation.

## Current Contract
- Contract version: ${contract.contractVersion}
- Stack packs: ${contract.stackPacks.map((pack) => pack.id).join(", ") || "none"}
- Foundation packs: ${contract.foundationPacks.map((pack) => pack.id).join(", ") || "none"}
`;
}

export function renderGitHubLabelerConfig(): string {
  return `domain:
- changed-files:
  - any-glob-to-any-file:
    - src/domain/**

tools:
- changed-files:
  - any-glob-to-any-file:
    - src/tools/**

schemas:
- changed-files:
  - any-glob-to-any-file:
    - src/tools/schemas/**

tests:
- changed-files:
  - any-glob-to-any-file:
    - tests/**

docs:
- changed-files:
  - any-glob-to-any-file:
    - README.md
    - docs/**
    - llms.txt
    - AGENTS.md

examples:
- changed-files:
  - any-glob-to-any-file:
    - examples/**

packs:
- changed-files:
  - any-glob-to-any-file:
    - packs/**
    - foundation-packs/**
    - policy-bundles/**
    - stack-sources/**

github:
- changed-files:
  - any-glob-to-any-file:
    - .github/**

dependencies:
- changed-files:
  - any-glob-to-any-file:
    - package.json
    - package-lock.json
`;
}

export function renderGitHubLabelerWorkflow(): string {
  return `name: Label pull requests

on:
  pull_request_target:
    types: [opened, synchronize, reopened, ready_for_review]

permissions:
  contents: read
  pull-requests: write

jobs:
  label:
    runs-on: ubuntu-latest
    steps:
      # This workflow uses pull_request_target only for labeling. Do not checkout or run PR code here.
      - uses: actions/labeler@v5
        with:
          repo-token: \${{ secrets.GITHUB_TOKEN }}
`;
}

export function renderGitHubCiWorkflow(contract: ArchitectureContract, brief?: ProjectBrief): string {
  const commands = verificationCommandsFor(contract, brief);
  const setupSteps = renderCiSetupSteps(commands);
  const verificationSteps = commands.length
    ? commands.map((command) => `      - run: ${command}`).join("\n")
    : "      - run: echo \"No verification command was specified; update this workflow before relying on CI.\" && exit 1";

  return `name: CI

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
${setupSteps}${verificationSteps}
`;
}

export function renderPullRequestTemplate(contract: ArchitectureContract, brief?: ProjectBrief): string {
  const commands = verificationCommandsFor(contract, brief);
  const verificationChecklist = commands.length
    ? commands.map((command) => `- [ ] \`${command}\``).join("\n")
    : "- [ ] Define and run the concrete verification command for this project";

  return `## Summary
- 

## Verification
${verificationChecklist}

## MCP Review
- [ ] Ran architect-mcp against this repo or explained why it was not relevant
- [ ] Addressed MCP findings or listed accepted residual risk

## Handoff
- Assumptions:
- Not done:
- Follow-up:
`;
}

export function verificationCommandsFor(contract: ArchitectureContract, brief?: ProjectBrief): string[] {
  const candidates = [
    ...(brief?.verification ?? []),
    ...contract.testingExpectations
  ];
  return extractVerificationCommands(candidates);
}

function renderCiSetupSteps(commands: string[]): string {
  const packageManager = packageManagerForVerificationCommands(commands);
  if (!packageManager) return "";
  if (packageManager === "bun") {
    const install = commands.includes("bun install --frozen-lockfile") ? "" : "      - run: bun install --frozen-lockfile\n";
    return `      - uses: oven-sh/setup-bun@v2
${install}`;
  }

  const installCommand = packageManager === "npm"
    ? "npm ci"
    : packageManager === "pnpm"
      ? "pnpm install --frozen-lockfile"
      : "yarn install --immutable";

  const corepack = packageManager === "pnpm" || packageManager === "yarn" ? "      - run: corepack enable\n" : "";
  const setupNode = `      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: ${packageManager}
`;
  const install = commands.includes(installCommand) ? "" : `      - run: ${installCommand}\n`;
  return `${setupNode}${corepack}${install}`;
}
