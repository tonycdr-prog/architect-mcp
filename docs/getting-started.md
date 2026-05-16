# Getting Started

architect-mcp is a local-first MCP server for coding-agent work gates. It asks better intake questions, creates implementation contracts, reviews plans before edits, checks drift after edits, and requires verification evidence before final output.

## Install From npm

The npm registry name `architect-mcp` is owned by a different package, so this project publishes as `@tonycdr-prog/architect-mcp` while keeping the `architect-mcp` command name.

```bash
npm install -g @tonycdr-prog/architect-mcp
architect-mcp
```

## Run From Source

```bash
npm install
npm run build
node dist/index.js
```

## Run The TUI

The optional terminal client is `architect-mcp-tui`. From a source checkout:

```bash
npm run tui:build
node bin/architect-mcp-tui.cjs
```

For scriptable automation:

```bash
architect-mcp-tui run --prompt "Build an offline recipe planner" --adapter codex --jsonl
```

## Default Surface

The default MCP surface exposes only the eight core work-gate tools:

- `grill_me`
- `create_pre_edit_contract`
- `review_build_plan`
- `review_proposed_file_plan`
- `review_repo_structure`
- `review_implementation_against_contract`
- `review_agent_final_response`
- `review_agent_session`

## Advanced Surface

Use the advanced surface when you need stack packs, standards intelligence, guarded MCP install plans, governance evals, operating-model evals, productization boundary evals, repo-quality gates, or local workspace utilities.

```bash
ARCHITECT_MCP_TOOL_SURFACE=advanced architect-mcp
```

The historical V3-V10 labels remain in filenames, tool names, scripts, and tests for compatibility. Public docs describe those areas as advanced maturity criteria rather than public product versions.

## Verify The Checkout

Before releasing or making broad tool-surface changes, run:

```bash
npm run release:check
```

That includes `npm run rust:check` for the TUI workspace.

For normal development, run the narrower checks that match the change:

```bash
npm run typecheck
npm test
npm run build
```
