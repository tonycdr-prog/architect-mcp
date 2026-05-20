# Use architect-mcp On A Repo

The default MCP surface is the agent work gate: `grill_me`, `create_pre_edit_contract`, `review_build_plan`, `review_proposed_file_plan`, `review_repo_structure`, `review_implementation_against_contract`, `review_agent_final_response`, and `review_agent_session`. Set `ARCHITECT_MCP_TOOL_SURFACE=advanced` when you need stack-pack authoring, governance, repo-quality, productization boundary, eval, repo constitution, workspace review, or MCP config review tools. Local-only tools still require an environment that enables trusted workspace access.

## Local Flow

1. Run `grill_me` with the project brief until blockers are gone.
2. Generate a contract with `generate_architecture_contract`.
3. Generate repo artifacts with `generate_repo_artifacts`.
4. Derive repo expectations with `derive_local_repo_constitution`, or use `derive_repo_constitution` when the client supplies file summaries, artifacts, and recent merged PR bodies.
5. Scan the workspace with `review_local_workspace` or provide file summaries to `review_repo_structure`.
6. Normalize review findings, external tool findings, verification summaries, coverage, and repo constitution output with `normalize_foundry_evidence` before scoring or routing.
7. Score normalized evidence with `score_foundry_actionability` before generating PR previews, issue previews, exceptions, or human questions.
8. Run `scan_mcp_config_files` to check `.mcp.json`, Cursor, Claude Desktop, and Codex MCP configs.
9. Use `review_build_plan` before implementation slices.
10. Use `review_agent_session` before final output when a client has intent, contract, changed files, verification, memory, and response data.
11. Use `review_agent_final_response` before sending a final reply.

## Hosted-Safe Flow

1. Do not use local file paths.
2. Send explicit file summaries to `review_repo_structure`.
3. Send repo constitution artifacts and recent merged PR summaries to `derive_repo_constitution`.
4. Send supplied review reports, external finding summaries, verification summaries, and constitution output to `normalize_foundry_evidence`.
5. Send normalized evidence to `score_foundry_actionability`; treat the result as advisory until the decision-ledger routing step records an explicit route.
6. Send parsed MCP config objects to `review_mcp_config_security`.
7. Use `audit_hosted_tool_policy` to confirm local-only tools are not exposed.
8. Keep memory tools stateless unless a future adapter is explicitly configured.

## Repo Constitution Flow

Use repo constitution output before drafting issues, PR previews, or mutation plans. The output is public-safe by design: it reports paths, counts, headings, package metadata, workflows, release signals, findings, and source provenance, but it does not return raw artifact bodies and it does not mutate files.

PR templates are hard repository signals. Recent merged PR bodies are advisory style evidence only: they can fill gaps when templates are missing, hidden-comment-only, too sparse, or possibly stale because accepted PR headings do not overlap the template. Recent PR style does not override explicit repo instructions, security policy, or template requirements. Bot-only PR bodies are ignored by default so dependency automation does not redefine accepted contribution style for normal PRs.

## Foundry Evidence Flow

Use `normalize_foundry_evidence` after repo constitution derivation and read-only audits, before actionability scoring or decision routing. The inventory assigns stable evidence IDs, source refs, confidence, public-safety class, redaction state, suppression candidates, merged coverage caveats, and finding histograms. It omits raw external payloads and raw repo content, redacts local paths and token-shaped values, and keeps mutation disabled.

Use `score_foundry_actionability` on that inventory before proposing maintainer work. The score is deterministic and advisory: it labels findings as `pr_preview_candidate`, `ask_human`, `exception_candidate`, or `no_op_candidate` and explains evidence strength, confidence, blast radius, patch-size confidence, maintainer fit, duplicate risk, release impact, verification path, public-safety risk, and expected maintainer value. The scorer does not mutate repositories and does not create issues or PRs; later routing and forge steps must still record an explicit decision.

## Baseline Flow

1. Run `review_repo_structure` in `audit` mode for a first pass on an established repo.
2. Create a baseline with `create_review_baseline`.
3. Store accepted findings with reasons.
4. Run later reviews in `ci` mode so new findings fail without blocking historical debt.

Use `migration` mode when adopting architect-mcp in a mature repo that already has known debt and you want lower-value line-count warnings suppressed in the report. Use `audit` mode when you want source findings without requiring `AGENTS.md` or `docs/architecture-contract.md` to exist yet. For large repos, check `report.coverage`: it includes total finding histograms before detailed-output suppression, scan truncation caveats, and the top scanned directories so capped audits are not mistaken for full-repo coverage.
