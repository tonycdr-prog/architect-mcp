# Rust TUI

`architect-mcp-tui` is a Ratatui and crossterm terminal client for architect-mcp. It keeps the TypeScript MCP server as the work-gate source of truth, then adds a local UI and headless runner for intake, plan review, adapter readiness, and guarded agent execution.

## Install

The npm package exposes the TUI as a separate binary:

```bash
npx -y --package @tonycdr-prog/architect-mcp architect-mcp-tui
```

The shim never downloads during `postinstall`. It first runs a local built binary when one is present and exposes the current required CLI commands. Stale local binaries that do not support the current smoke surface are skipped. If no usable local binary is found, the shim downloads the matching GitHub release binary into a user cache and verifies the `.sha256` file before execution.

For a source checkout:

```bash
npm install
npm run tui:build
node bin/architect-mcp-tui.cjs
```

## CLI

Interactive TUI:

```bash
architect-mcp-tui
```

The command palette supports the guarded app-building path:

```text
new app <idea>
answer users=home cooks need controlled agent help
answer coreFlows=grill brief; review plan; promote reviewed changes
answer stack=frontend=Rust Ratatui; backend=TypeScript MCP
answer verification=cargo test --workspace; npm run release:check
grill
contract
review plan
review files
approve run isolated adapter
run adapter
diff summary
diff file <changed-path>
verification status
record verification <required-check>=passed
final review <response>
session review
promotion status
approve promote reviewed diff
promote
promotion receipt
arena run codex,shell
arena rank
```

Sessions are persisted under `.architect-mcp/tui/sessions/<id>.json` without secrets.

Use `answer key=value` to fill grill blockers before rerunning `grill`. List-like fields accept semicolon-separated values, for example `coreFlows=grill; review; promote` and `verification=cargo test; npm run release:check`. Stack and repo layout answers accept key pairs, for example `stack=frontend=Rust Ratatui; backend=TypeScript MCP` and `repoLayout=tui=crates/architect-tui/src; docs=docs`.

`approve` is phase-aware. After `review files`, it approves adapter execution only. After adapter evidence, implementation review, and session review are recorded, it approves promotion. Execution approval is cleared after the adapter run, so promotion still needs a separate approval. Use `promotion status` before approving or promoting to see every blocker and next action: missing approval, missing isolated-worktree evidence, missing changed-file evidence, failed verification, missing review gates, or blocking review output. Successful promotion records a compact session receipt with the approval or override reason, promoted files, changed-file evidence, review gate state, verification state, and adapter-run issue summary. Use `promotion receipt` after promotion, including after `resume <session-id>`, to inspect that compact receipt without exposing raw adapter logs or terminal output.

Verification evidence is strict. The TUI captures the required checks from the live `grill_me` and build-plan gates. Use `verification status` to list the required checks, then use `record verification <check>=passed` with an exact required check name before final review, session review, or promotion approval can proceed. Failed, skipped, not-run, missing, unknown-check, and unknown-status records block the normal path; `override <reason>` remains the explicit maintainer escape hatch and requires a maintainer-written reason.

Adapter run evidence is also strict. Timeout, crash, cancellation, non-zero exit, and truncated output are saved into the session as adapter issues, shown in the inspector, and block normal promotion approval/readiness until the adapter is rerun successfully or a maintainer records an explicit override. A rerun requires file-plan review and execution approval again, replaces the managed isolated worktree for that session/adapter, clears stale run evidence, and then applies the new evidence.

Headless JSONL run:

```bash
architect-mcp-tui run --prompt "Build an offline recipe planner" --adapter codex --jsonl
```

Headless mode is gate-only by default. It calls live architect-mcp tools and stops before adapter execution unless `--execute` is supplied:

```bash
architect-mcp-tui run --prompt "Build an offline recipe planner" --adapter codex --jsonl --execute
```

ACP stdio server:

```bash
architect-mcp-tui acp --stdio
```

ACP mode is currently a provisional JSON-RPC compatibility surface for early client testing, not a full ACP conformance claim. It fails closed on malformed request envelopes before method dispatch: messages must be JSON-RPC 2.0 objects, request ids must be strings, numbers, or null, and request methods must be non-blank strings. Valid notifications still produce no response. The session boundary is also strict: clients can only create sessions with configured adapters, supported modes, known session parameters, concurrency in the advertised range, and the configured worktree-isolation and approval policy. Session-scoped methods reject non-object params, unknown fields, missing `sessionId`, wrong `sessionId` types, and blank prompt turns before reading or mutating session state.

Config commands:

```bash
architect-mcp-tui config init
architect-mcp-tui config doctor
architect-mcp-tui config adapters
architect-mcp-tui config adapters --json
```

Terminal QA smoke:

```bash
architect-mcp-tui smoke --json
```

The smoke command checks help output, adapter readiness, binary SHA-256 and cache metadata, a secret-safe environment summary, and a live gate-only JSONL run. See [Terminal QA](./terminal-qa.md) for platform-specific commands.

Scripted interactive walkthrough:

```bash
architect-mcp-tui walkthrough --json
```

The walkthrough runs the same command-palette engine as the interactive TUI in a throwaway git workspace. It creates a session, answers intake, runs grill/contract/plan/file gates, approves and runs a fixture adapter in an isolated worktree, inspects diff evidence, records verification, runs final/session review, checks `promotion status`, approves promotion, and promotes the reviewed file. It is reproducible release evidence for the operator flow, but it does not replace manual visual terminal QA.

Local real-adapter promotion smoke:

```bash
architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace
```

This command is explicit local QA, not a default CI release gate. It creates a disposable git repo, wires the TUI to the source architect-mcp server, confirms the selected adapter is ready, drives the command-palette engine through grill, contract, plan review, file-plan review, execution approval, isolated adapter execution, diff inspection, verification, final/session review, promotion approval, and promotion. The default Codex path asks for a documentation-only change to `docs/codex-adapter-smoke.md`; the smoke passes only when that file is the only promoted file. Keep the workspace when collecting evidence so the isolated worktree, promoted file, and session JSON can be inspected.

Repo-foundry smoke:

```bash
architect-mcp-tui foundry-smoke --owner <github-owner> --json
architect-mcp-tui foundry-smoke --owner <github-owner> --repo <repo-name> --execute --confirm-private-repo-mutation --json --keep-workspace
```

Without `--execute`, this drives the command-palette work gate through grill, contract, plan review, file-plan review, private repo planning, local staging, and create preview only. With `--execute`, it first requires `--confirm-private-repo-mutation`, refuses to reuse an existing GitHub repo, creates a private repo, pushes `main` and `architect/bootstrap`, opens the first draft PR, verifies the repo is private, and reports the retained evidence URLs.

## Work Gate

Every coding and app-building loop starts with the architect-mcp work gate:

1. `grill_me`
2. `create_pre_edit_contract`
3. `review_build_plan`
4. `review_proposed_file_plan`
5. adapter execution only after explicit approval or `--execute`
6. `review_implementation_against_contract`
7. verification evidence
8. `review_agent_final_response`
9. `review_agent_session`

The headless runner starts architect-mcp with `ARCHITECT_MCP_TOOL_SURFACE=advanced`, calls `grill_me` over stdio first, and stops with `approval_required` when the brief is incomplete. If the brief is ready, it calls `create_pre_edit_contract`, `review_build_plan`, and `review_proposed_file_plan`, then pauses before any adapter process unless `--execute` is present.

## Interface

The main layout has four surfaces:

- Left: session and agent tree.
- Center: transcript and plan panel.
- Right: inspector for gates, adapters, and approval state.
- Bottom: command palette and prompt input.

On narrow terminals, the inspector moves into a full-width band above the command palette so Agents, Transcript, Inspector, and Command remain visible at the common 80x24 terminal size.

Mouse capture supports layout-aware click, drag, scroll, tab switching, and agent pinning. Approval and promotion are command-palette actions: use `diff summary` and `diff file <path>` to inspect recorded isolated-worktree changes, use `promotion status` to inspect blockers, use `approve [reason]` after review gates pass, then `promote` to copy approved isolated-worktree files back into the workspace. Promotion requires changed-file evidence plus implementation, repo-structure, final-response, and session review gates unless `override <reason>` is used. Overrides require an explicit maintainer reason and still cannot bypass isolated-worktree or changed-file evidence. The persisted session inspector reports whether a promotion receipt exists; `promotion receipt` shows the compact receipt fields: decision, reason, promoted files, changed-file evidence count, review gate state, verification state, and adapter issue count. The receipt is compact metadata, not raw adapter output. The TUI never promotes adapter output automatically.

The render scheduler coalesces redraw requests and relies on Ratatui backend diffing instead of clearing the screen after startup. It does not perform true widget-level partial painting.

## Adapters

Built-in adapter templates are Codex, Claude, Gemini, OpenCode, Aider, and a generic shell adapter. Adapters are runtime-probed and show as unavailable when the local CLI is missing. Codex also reports auth state from `codex login status`; it is ready only when the command succeeds and reports `Logged in`. The default Codex template runs `codex exec --sandbox workspace-write --color never --ephemeral` so approved TUI execution uses a bounded non-interactive Codex run inside the isolated worktree instead of launching the interactive Codex UI. It does not request Codex JSONL by default because verbose event streams can exceed the TUI output cap and become promotion blockers even when the diff and verification evidence are valid. Other authenticated CLIs remain `auth unknown` until reliable probes are added.

Agents run in a PTY by default when execution is explicitly enabled. Headless `--execute` creates an isolated git worktree, sends the adapter the original request plus the approved pre-edit contract, required verification checks, and execution rules, streams PTY output, records changed-file evidence, and runs `review_implementation_against_contract`, `review_repo_structure`, `review_agent_final_response`, and `review_agent_session` before any promotion. PTY output is capped with an explicit truncation marker, and truncated output becomes a promotion blocker in the normal path.

The `arena run <adapter[,adapter]>` command requires file-plan review plus explicit execution approval, then runs two or more named adapters against the current contract in isolated worktrees and records each candidate's diff evidence. The `arena rank` command ranks recorded candidates with a deterministic score based on implementation review status, verification status, diff size, contract drift, and crash state. Use `arena select <adapter>` to move one recorded candidate into the normal verification, final/session review, approval, and promotion path. Crashed, timed-out, cancelled, or otherwise failed candidates cannot be selected for the normal promotion path. Selection does not copy files by itself; candidate promotion remains manual and the arena never auto-merges a winner.

## Integrations

The command palette exposes the MCP catalog flow through guarded `integrations` commands:

```text
integrations recommend [extra context]
integrations plan <server-id> [target-client]
integrations review
integrations apply [target-path]
integrations approve <reason>
integrations write [target-path]
```

`integrations recommend` calls the live `recommend_mcp_servers` tool with the current brief and current clarified answers instead of shortcutting from the original prompt text. Generic database needs ask for a provider before Supabase can be planned. `integrations plan` is blocked unless the server was recommended by the latest recommendation result. `integrations review` must pass before apply, approval, or write. `integrations apply` is dry-run only and does not write files. `integrations write` calls `apply_mcp_install_plan` with `writeFiles=true` only after `integrations approve <reason>`, and the write approval is consumed after a successful write. Session persistence keeps recommendation/plan/review metadata, not raw MCP config payloads or credentials.

## Repo Foundry

The command palette exposes an early repo-foundry planning flow for new app sessions:

```text
foundry plan <repo-name> [owner=name]
foundry status
foundry approve <reason>
foundry stage
foundry create
foundry create --execute
```

`foundry plan` is available only after `review files` has passed. It creates a private-by-default GitHub repo plan with required scaffold artifacts for `AGENTS.md`, `README.md`, `.env.example`, architecture and build-plan docs, CI, issue template, and PR template. When the verified plan includes `npm test`, the staged repo also includes a minimal `package.json` and scaffold test so generated CI has a real executable check. `foundry approve <reason>` is a separate approval from adapter execution or promotion approval.

`foundry stage` requires approval and materializes the generated app scaffold into `.architect-mcp/foundry/<session>/<repo>` as a separate git repository with `main` and `architect/bootstrap` branches. Staging consumes the approval, so live GitHub creation requires another explicit `foundry approve <reason>`.

`foundry create` without flags renders a preview and performs no GitHub mutation. `foundry create --execute` requires a staged repo plus fresh approval, then runs the private `gh repo create`, pushes `main` and `architect/bootstrap`, and opens a draft PR using the staged evidence body. Changing clarified answers clears stale foundry state so repo plans cannot silently survive a changed brief.

`architect-mcp-tui foundry-smoke --owner <github-owner> --json` is the repeatable dry-run QA path for the repo-foundry flow. Live GitHub creation is intentionally noisier: `--execute --confirm-private-repo-mutation` must both be present, and the command fails closed if the target repository already exists.

Governance audit:

```bash
architect-mcp-tui governance-audit --json
```

This command is read-only. It checks the current workspace for agent instructions, architecture/build-plan docs, package scripts, lockfiles, CI and release gates, environment templates, memory-policy safety, and secret-shaped local config. It separates deterministic release gates from smoke evidence, proposes only memory-safe durable context, and calls `review_local_workspace` in `mode=audit` when architect-mcp is available. Use `--skip-mcp` only when collecting static evidence without a live MCP process. See [Governance Audit](./governance-audit.md) for the recurring workflow and public-safe issue evidence form.

Launch judge:

```bash
architect-mcp-tui launch-judge --json
architect-mcp-tui launch-judge --json --run-release-check --require-clean-git
architect-mcp-tui launch-judge --json --terminal-evidence terminal-evidence.json
architect-mcp-tui launch-judge --json --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui launch-judge --public-summary --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
```

This command combines the read-only governance audit, terminal smoke, release-gate evidence, git worktree state, and external terminal evidence into one `go`, `conditional_go`, or `no_go` report. By default it does not run `npm run release:check`; missing release-gate execution is a `conditional_go` warning with a next action. Use `--run-release-check` when collecting release-sensitive evidence. Use `--skip-mcp` or `--skip-smoke` only when documenting why the judge report is intentionally partial. A `no_go` result exits non-zero; `conditional_go` exits zero so maintainers can inspect incomplete external evidence without breaking static CI.

Use `--public-summary` for public PR comments, release notes, and issue evidence. It emits a compact launch decision summary with check status, next actions, release-gate attempt/result, terminal-evidence source filenames, and terminal-evidence platform status. It intentionally omits the workspace path, governance-audit internals, smoke report internals, raw command tails, terminal-evidence freeform source text, command summaries, notes, stdout/stderr logs, cache paths, private repo names, and token-shaped values. Keep `--json` for local diagnosis when you need the full report.

Launch stack:

```bash
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --pr 150 --pr 178 --blocker 136
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136
architect-mcp-tui launch-stack --json --repo tonycdr-prog/architect-mcp --pr 150 --pr 178 --blocker 136 --waive-blocker 136="maintainer accepted a temporary platform waiver with reason"
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --public-summary --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136 --waive-blocker 136="maintainer accepted a temporary platform waiver with reason" --waive-terminal-evidence 136="maintainer accepted launch without manual Linux/Windows terminal evidence"
architect-mcp-tui evidence-index --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --markdown-output .architect-mcp/release/evidence-index.md --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui evidence-index --json --require-go --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
```

`launch-stack` complements `launch-judge`; it does not replace the clean checkout release gate or terminal evidence. It uses GitHub CLI read-only lookups to summarize explicit PR numbers and external blocker issue numbers into `go`, `conditional_go`, or `no_go`. Use `--stack-from-pr <number>` when the release is a stacked PR chain: the command follows each open PR's base branch to another open PR head branch, reports the discovered base-to-head merge order, and stops when it reaches a non-PR base such as `main`. Failed PR checks or dirty/unknown merge states are `no_go`; draft PRs, pending checks, temporarily unstable merge states caused by pending checks, and open blocker issues are `conditional_go`. Maintainers can make a waiver explicit with `--waive-blocker ISSUE=reason`; waived blockers are reported as `waived` with the public reason and only contribute to `go` when all other stack checks pass. The launch-stack JSON output now uses `schemaVersion: 2` so parsers can detect waiver fields (`status: "waived"` and optional `waiverReason`). The output is public-safe and omits raw workflow logs, local paths, token-shaped values, and repository-local diagnostics.

`launch-readiness` combines `launch-stack` with a read-only terminal-evidence issue scan. Use it for the final maintainer view of a stacked release: discovered or explicit PRs, explicit open blockers, waiver status, extracted terminal evidence, and next actions in one report. Add `--public-summary` when posting that decision publicly; it keeps counts, PR order, blocker state, evidence platforms, waiver state, findings, and next actions without dumping full PR/check payloads. It does not turn a waiver into terminal evidence; missing or unsafe Linux/Windows evidence remains visible in the terminal-evidence section. `--waive-terminal-evidence ISSUE=reason` is allowed only for the matching terminal-evidence issue, is recorded separately in the report, and can only waive missing or incomplete evidence. Malformed or unsafe evidence remains `no_go`.

`evidence-index` is the public release handoff view. It runs the launch-readiness rollup and the read-only governance audit, then emits one public-safe report with the combined `go`, `conditional_go`, or `no_go` result, section summaries, waiver visibility, findings, and next actions. Use `evidence-index --json` for machine-readable release gates and `evidence-index --markdown` for PR comments, release notes, and goal-ledger updates. Use `evidence-index --markdown-output <path>` to write the same Markdown handoff to a relative workspace path such as `.architect-mcp/release/evidence-index.md`; the command creates parent directories, rejects absolute or parent-traversal paths, and does not post to GitHub. Add `evidence-index --require-go` only for strict launch scripts that should fail unless the combined result is exactly `go`; default handoff mode still exits zero for `conditional_go` so maintainers can publish incomplete evidence and next actions. The index reuses the public summaries and omits workspace paths, raw command strings, raw memory proposal text, raw MCP payloads, full PR/check payloads, terminal-evidence freeform source text, command summaries, notes, stdout/stderr tails, private repo names, local paths, and token-shaped values. A `no_go` index exits non-zero after printing or writing the report, and `--require-go` also exits non-zero for `conditional_go`. `--json` and `--markdown` are mutually exclusive; `--markdown-output` is a side artifact and can be paired with either stdout mode.

Linux and Windows testers can generate the public-safe evidence file directly:

```bash
architect-mcp-tui terminal-evidence --markdown
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
architect-mcp-tui terminal-evidence --json --platform linux > terminal-evidence.json
```

`terminal-evidence` runs the terminal smoke internally, maps the smoke result into the launch judge evidence schema, fills `collectedAt` with the current UTC date, and omits raw JSONL, stdout/stderr logs, local binary paths, cache paths, private repo names, and token-shaped values. Use `--markdown` for a ready-to-paste GitHub issue comment with fenced terminal-evidence JSON; the command does not create, edit, or close GitHub issues. Use `--json` when saving a local file for `launch-judge --terminal-evidence`. It auto-detects Linux and Windows; use `--platform` only when summarizing already-verified external evidence or when a maintainer is exercising the command surface outside a launch platform. Use `--collected-at YYYY-MM-DD` only when summarizing already-verified external evidence from a known date; other values fail without echoing the raw input.

`--terminal-evidence` accepts generated public-safe JSON summary files for manual Linux and Windows terminal QA. Pass it more than once when Linux and Windows reports were generated on separate machines; the launch judge merges the reports and validates the combined evidence. It is intentionally not a raw log importer: files with `stdout`, `stderr`, raw log fields, absolute local paths, or secret-shaped strings fail closed. Use short summaries and public issue or PR references:

```bash
architect-mcp-tui collect-terminal-evidence --json --repo tonycdr-prog/architect-mcp --issue 136
```

`collect-terminal-evidence` is a read-only helper for public GitHub issue reports. It extracts fenced terminal-evidence JSON, rejects unsafe blocks, reports missing or duplicated platforms, and prints a merged evidence envelope that can be saved for `launch-judge --terminal-evidence`.

Template placeholders are treated as incomplete evidence. Reports that still contain `REPLACE with ...`, `issue #136 public-safe terminal QA report`, or the default issue-template command summary keep the launch judge at `conditional_go` until the reporter supplies real platform-specific results.

```json
{
  "schemaVersion": 1,
  "reports": [
    {
      "platform": "linux",
      "status": "passed",
      "source": "issue #136 public-safe summary",
      "commandSummary": "architect-mcp-tui help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "summary only, no raw logs"
    },
    {
      "platform": "windows",
      "status": "passed",
      "source": "issue #136 public-safe summary",
      "commandSummary": "architect-mcp-tui help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "summary only, no raw logs"
    }
  ]
}
```

## Config

Repo config lives at:

```text
.architect-mcp/tui.toml
```

User config lives at:

```text
~/.config/architect-mcp/tui.toml
```

Repo config may override adapters only inside the workspace. Secrets must come from environment variables, not config values.

## Release Gate

The release gate now includes the Rust checks before the existing maturity gate:

```bash
npm run release:check
```

Rust-only checks:

```bash
npm run rust:check
```

That runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.

Cross-platform install smoke, live-QA smoke, and scripted interactive walkthrough checks run in GitHub Actions on Linux, macOS, and Windows. Manual terminal checks are described in [Terminal QA](./terminal-qa.md), and release-candidate evidence is tracked in [TUI Live QA](./tui-live-qa.md).
