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

${body}

## Proof Of Completion
- Run the relevant tests or typecheck command for the files changed.
- Run architect-mcp repo review after scaffolding or large feature changes.
- Run or emulate review_agent_final_response before final replies so changed, verified, assumptions, skipped checks, and not-done items are explicit.
- Explain any intentional rule violations in the final response.
`;
}
