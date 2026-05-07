import { renderContractMarkdown } from "./contract.js";
import type { ArchitectureContract } from "./types.js";

export type AgentInstructionFormat = "agents-md" | "claude-md" | "cursor-rules";

export function renderAgentInstructions(contract: ArchitectureContract, format: AgentInstructionFormat): string {
  const body = renderContractMarkdown(contract);

  if (format === "cursor-rules") {
    return `---
description: Standards, architecture, and verification guardrails for AI-generated implementation work
alwaysApply: true
---

${body}`;
  }

  const title = format === "claude-md" ? "# CLAUDE.md" : "# AGENTS.md";

  return `${title}

This repository uses architect-mcp as a local-first standards, architecture, and verification harness for coding agents. Follow these rules before making implementation changes.

## Project Context
- Purpose: ${contract.purpose}
- Stack: ${Object.values(contract.stack).filter(Boolean).join(", ") || "not specified"}
- Contract version: ${contract.contractVersion}

## Setup Commands
- Install dependencies using the package manager already present in the repo.
- Prefer exact repo scripts such as \`npm run typecheck\`, \`npm test\`, and \`npm run build\` when they exist.

## Testing Instructions
- Run the narrowest meaningful test first, then run the broader verification command before completion.
- Treat skipped, failed, or unavailable checks as not-done evidence, not success.

## Build Order For Coding Agents
1. Run /grill-me and stop if any blocker remains.
2. Generate or update AGENTS.md, docs/architecture-contract.md, and editor rules.
3. Create scaffold directories by responsibility before feature code.
4. Build one workflow slice at a time.
5. Run verification and architecture review before starting the next slice.

## Do Not Create
- A giant App.tsx, page.tsx, server.ts, route.ts, or index.ts containing multiple workflows.
- UI files that own database access, server secrets, or migration logic.
- Shared folders without at least two real consumers.
- Backend routes/controllers that own business workflows instead of calling services/use-cases.
- Database schema changes without migration ownership.

## Code Boundaries
- Keep workflow logic inside feature, service, repository, or adapter modules named by responsibility.
- App entry files compose providers and routes; they do not own business workflows, database access, auth policy, or large state orchestration.

${body}

## Proof Of Completion
- Run the relevant tests or typecheck command for the files changed.
- Run architect-mcp repo review after scaffolding or large feature changes.
- Run or emulate review_agent_final_response before final replies so changed, verified, assumptions, skipped checks, and not-done items are explicit.
- Explain any intentional rule violations in the final response.
`;
}
