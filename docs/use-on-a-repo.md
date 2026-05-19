# Use architect-mcp On A Repo

The default MCP surface is the agent work gate: `grill_me`, `create_pre_edit_contract`, `review_build_plan`, `review_proposed_file_plan`, `review_repo_structure`, `review_implementation_against_contract`, `review_agent_final_response`, and `review_agent_session`. Set `ARCHITECT_MCP_TOOL_SURFACE=advanced` when you need stack-pack authoring, governance, repo-quality, productization boundary, or eval tools.

## Local Flow

1. Run `grill_me` with the project brief until blockers are gone.
2. Generate a contract with `generate_architecture_contract`.
3. Generate repo artifacts with `generate_repo_artifacts`.
4. Scan the workspace with `review_local_workspace` or provide file summaries to `review_repo_structure`.
5. Run `scan_mcp_config_files` to check `.mcp.json`, Cursor, Claude Desktop, and Codex MCP configs.
6. Use `review_build_plan` before implementation slices.
7. Use `review_agent_session` before final output when a client has intent, contract, changed files, verification, memory, and response data.
8. Use `review_agent_final_response` before sending a final reply.

## Hosted-Safe Flow

1. Do not use local file paths.
2. Send explicit file summaries to `review_repo_structure`.
3. Send parsed MCP config objects to `review_mcp_config_security`.
4. Use `audit_hosted_tool_policy` to confirm local-only tools are not exposed.
5. Keep memory tools stateless unless a future adapter is explicitly configured.

## Baseline Flow

1. Run `review_repo_structure` in `audit` mode for a first pass on an established repo.
2. Create a baseline with `create_review_baseline`.
3. Store accepted findings with reasons.
4. Run later reviews in `ci` mode so new findings fail without blocking historical debt.

Use `migration` mode when adopting architect-mcp in a mature repo that already has known debt and you want lower-value line-count warnings suppressed in the report. Use `audit` mode when you want source findings without requiring `AGENTS.md` or `docs/architecture-contract.md` to exist yet. For large repos, check `report.coverage`: it includes total finding histograms before detailed-output suppression, scan truncation caveats, and the top scanned directories so capped audits are not mistaken for full-repo coverage.
