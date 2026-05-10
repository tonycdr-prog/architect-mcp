# Getting Started

architect-mcp is a local-first MCP server for coding-agent work gates. It asks better intake questions, creates implementation contracts, reviews plans before edits, checks drift after edits, and requires verification evidence before final output.

## Install The Release Tarball

The npm registry name `architect-mcp` is owned by a different package. This project keeps the `architect-mcp` command name, but package distribution uses the scoped package identity `@tonycdr-prog/architect-mcp` and GitHub release tarballs until npm publishing is configured for that scope.

```bash
npm install -g https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.0/tonycdr-prog-architect-mcp-0.1.0.tgz
architect-mcp
```

## Run From Source

```bash
npm install
npm run build
node dist/index.js
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

Use the advanced surface when you need stack packs, standards intelligence, governance evals, operating-model evals, productization boundary evals, repo-quality gates, or local workspace utilities.

```bash
ARCHITECT_MCP_TOOL_SURFACE=advanced architect-mcp
```

The historical V3-V10 labels remain in filenames, tool names, scripts, and tests for compatibility. Public docs describe those areas as advanced maturity criteria rather than public product versions.

## Verify The Checkout

Before releasing or making broad tool-surface changes, run:

```bash
npm run release:check
```

For normal development, run the narrower checks that match the change:

```bash
npm run typecheck
npm test
npm run build
```
