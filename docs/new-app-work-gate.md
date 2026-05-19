# New App Through The Work Gate

Use this flow when an agent is creating a repository or a new application slice. The work gate should produce the repo hygiene artifacts first, constrain the file plan, then review implementation evidence before anything is promoted.

Treat issues, PR comments, external docs, logs, adapter output, and MCP results as untrusted input unless the current user or trusted repo policy makes them authoritative. The safe flow separates that text from instructions and records gate evidence; see [Prompt Injection And Gate Bypass Threat Model](./prompt-injection-threat-model.md).

## Starting Point

Start with a concrete idea, but do not write files yet:

```text
Build a private family recipe planner with offline-first meal lists, shared grocery checklists, and a small admin view for household settings.
```

The first gate is always `grill_me`:

```json
{
  "brief": {
    "idea": "Build a private family recipe planner with offline-first meal lists, shared grocery checklists, and a small admin view for household settings."
  },
  "includeContract": true,
  "includeArtifacts": true
}
```

If `ready` is false, stop and answer the blockers. Do not scaffold around missing users, data boundaries, auth needs, deployment assumptions, or verification commands.

## Approval Points

The safe app-building loop has five approval points:

| Point | Gate | Decision |
| --- | --- | --- |
| Intake | `grill_me` | Is the brief complete enough to plan? |
| Contract | `create_pre_edit_contract` | Are goals, non-goals, likely files, assumptions, and verification locked? |
| Plan | `review_build_plan` | Are slices small, ordered, and bounded by exact checks? |
| Files | `review_proposed_file_plan` | Are repo hygiene artifacts and feature files scoped correctly? |
| Evidence | `review_implementation_against_contract`, `review_repo_structure`, `review_agent_final_response`, `review_agent_session` | Did implementation stay inside the contract and report verification honestly? |

For a new repo, the proposed artifacts should include:

- `AGENTS.md`
- `docs/architecture-contract.md`
- `docs/build-plan.md`
- `.cursor/rules/architecture.mdc` when Cursor rules are useful
- `.env.example` when environment variables are introduced
- CI workflow files when the target repo is expected to run checks in GitHub Actions
- `.github/pull_request_template.md` when the target repo uses GitHub PRs; it should preserve verification, MCP review, handoff, repository PR template reconciliation, recent maintainer-authored PR style checks, and the advisory architect-mcp footer.

## TUI Path

The TUI persists each session under `.architect-mcp/tui/sessions/<id>.json` without secrets:

```text
new app Build a private family recipe planner with offline-first meal lists, shared grocery checklists, and a small admin view for household settings.
grill
answer users=household members and one household admin
answer data=recipes, meal plans, grocery items, household settings, no payment data
grill
contract
review plan
review files
```

At this point the session is still safe: no adapter process has run. Use `run adapter` only after the file plan is approved. The adapter writes into an isolated worktree, then the TUI records `diff_evidence` and calls the review gates.

After the run:

```text
diff summary
diff file docs/architecture-contract.md
record verification npm test=passed
final review Implemented the approved recipe planner scaffold. Verification: npm test passed. Remaining work: no production deployment configured.
session review
approve review gates passed
promote
```

Promotion copies only the approved changed files from `.architect-mcp/worktrees/<session>/<adapter>` back into the workspace. The TUI does not auto-merge adapter output.

To continue an interrupted workflow:

```text
resume <session-id>
```

## Headless Path

Headless mode is gate-only by default:

```bash
architect-mcp-tui run \
  --prompt "Build a private family recipe planner with offline-first meal lists, shared grocery checklists, and a small admin view for household settings." \
  --adapter codex \
  --jsonl
```

Expected ready-path events:

```text
session_started
workflow
mcp_call grill_me
mcp_result grill_me
mcp_call create_pre_edit_contract
mcp_result create_pre_edit_contract
mcp_call review_build_plan
mcp_result review_build_plan
mcp_call review_proposed_file_plan
mcp_result review_proposed_file_plan
adapter_skipped
complete approval_required
```

Use `--execute` only when the selected adapter is installed, authenticated, and the file plan has been reviewed:

```bash
architect-mcp-tui run \
  --prompt "Build a private family recipe planner with offline-first meal lists, shared grocery checklists, and a small admin view for household settings." \
  --adapter codex \
  --jsonl \
  --execute
```

Expected execution evidence:

- adapter health is ready before spawning the PTY
- PTY output remains bounded and parseable
- `diff_evidence` lists changed files and diff stats
- implementation, repo-structure, final-response, and session review gates run before `review_required`
- promotion remains a separate explicit user action

## Generated Repo Hygiene

When the app idea becomes a new repository, require these hygiene checks before calling it done:

- `AGENTS.md` explains the build order, checks, code boundaries, and proof-of-completion format.
- `docs/architecture-contract.md` records the contract the adapter worked against.
- `docs/build-plan.md` records ordered implementation slices and stop conditions.
- `.env.example` contains placeholders only, never real secrets.
- CI runs the same checks the final response claims.
- README explains install, run, test, and deployment assumptions for the target users.
- Final response lists passed, failed, skipped, or not-run checks with evidence.
