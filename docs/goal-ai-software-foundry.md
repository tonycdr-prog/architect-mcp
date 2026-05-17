# AI Software Foundry Goal

This document is the durable repo state for the evolved architect-mcp goal. The active runtime goal lives in Codex `/goal`; this page keeps the expanded spec, roadmap, issue tracker, judge protocol, and evidence log available across sessions.

## Objective

Evolve architect-mcp into the local-first control layer for AI-native software delivery.

It must turn rough product intent into governed software changes: clarify the brief, create a contract, review the build plan and file plan, run agents only with approval, inspect diffs, verify evidence, review drift, and produce honest final/session reviews.

The TUI is the operator surface. It must support new-app creation, repo audit, Codex and other adapters, isolated worktrees, multi-agent arena runs, MCP install recommendations, approval/promotion, release evidence, and hard go/no-go judging.

The human owns decisions. Agents propose and execute. architect-mcp governs.

## Tracking

- Epic: [#142 - Evolve architect-mcp into an AI software delivery control plane](https://github.com/tonycdr-prog/architect-mcp/issues/142)
- Active slice: [#143 - TUI full control loop: interactive gate to verified promotion](https://github.com/tonycdr-prog/architect-mcp/issues/143)
- Runtime goal: Codex CLI `/goal`, backed by this document and the GitHub epic.

Every implementation slice should have its own issue and PR. Every PR should link to the epic, list verification, record the judge result, and state remaining gaps.

## Roadmap

| Milestone | Issue | Status | Acceptance signal |
| --- | --- | --- | --- |
| TUI production control loop | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | Active | User can complete a TUI work-gate path to verified promotion without bypassing gates. |
| Adapter evidence and promotion hard gates | [#144](https://github.com/tonycdr-prog/architect-mcp/issues/144) | Planned | Adapter runs record evidence, failures cannot look successful, and promotion is review-gated. |
| Multi-agent arena hardening | [#145](https://github.com/tonycdr-prog/architect-mcp/issues/145) | Active | Multiple candidates run in isolated worktrees and are ranked without auto-promotion. |
| MCP catalog and install-plan flow | [#146](https://github.com/tonycdr-prog/architect-mcp/issues/146) | Planned | Recommendations require clarified need, dry-run install plans, security review, and approval before config writes. |
| New app to private repo foundry path | [#147](https://github.com/tonycdr-prog/architect-mcp/issues/147) | Planned | A clarified app idea can become a private repo with CI, docs, agent instructions, env template, and first PR evidence. |
| Governance audit and drift evidence loop | [#148](https://github.com/tonycdr-prog/architect-mcp/issues/148) | Planned | Maintained repos can be audited read-only for drift, stale docs, weak tests, unsafe config, and memory safety. |

Future hosted and team mode should stay behind the local-first proof. Hosted work is not launch-blocking for the local operator path, and hosted mode must keep local-only tools excluded.

## Operating Loop

Each slice should follow the agent work gate:

1. Run `grill_me` and stop while blockers remain.
2. Create or update the pre-edit contract.
3. Review the build plan.
4. Review the proposed file plan.
5. Run implementation only after approval.
6. Review implementation drift and repo structure.
7. Record verification evidence.
8. Review the final response and full session.
9. Issue a judge result before merge or launch claims.

For TUI work, the TUI should make these gates visible and enforceable. For docs or planning work, the PR body should still state the relevant checks and judge result.

## Judge Protocol

A slice is `go` only when:

- It advances the evolved spec.
- User approval is required before repo mutation, adapter execution, MCP install, promotion, merge, and release.
- Work-gate evidence exists where applicable: grill, contract, plan review, file-plan review, implementation review, verification, final/session review.
- Docs and public claims match actual behavior.
- Tests pass, or failures/skips are explicitly justified.
- `npm run release:check` passes for release-sensitive changes.

A slice is `conditional go` only when:

- Core behavior works.
- The remaining gap is documented.
- The gap does not block the next slice.
- The PR body and this document record the limitation.

A slice is `no-go` when:

- Behavior is simulated but documented as real.
- User control can be bypassed.
- Verification is missing or overstated.
- Hosted/local-only boundaries are unsafe.
- Release gates fail without a justified non-release scope.

## Evidence Log

| Date | Evidence | Result |
| --- | --- | --- |
| 2026-05-16 | [#141](https://github.com/tonycdr-prog/architect-mcp/pull/141) merged terminal QA smoke workflow after green CI, install-smoke, and live-QA workflows. | `go` as baseline terminal QA evidence. |
| 2026-05-16 | Codex CLI `/goal` accepted the concise evolved objective. | Runtime goal created; detailed spec lives in this document. |
| 2026-05-16 | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) structured-intake slice: TUI prompts and `answer key=value` commands now shape live `grill_me` briefs, generated build-plan checks are merged into contract verification, and a live headless ready prompt reached `grill_me`, `create_pre_edit_contract`, `review_build_plan`, and `review_proposed_file_plan` with passing review gates before stopping at adapter approval. Verification: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `architect-mcp-tui smoke --json`, and `npm run release:check`. | `conditional go`: intake and gate alignment work; full interactive adapter execution, diff review, verification capture, session review, and promotion remain for later #143/#144 slices. |
| 2026-05-16 | [#151](https://github.com/tonycdr-prog/architect-mcp/pull/151) stacked TUI execution-approval slice: `run adapter` now requires explicit execution approval after file-plan review, execution approval is cleared after a successful adapter run, and promotion still requires a separate approval after adapter review evidence. Verification: `cargo test --workspace` and `npm run release:check`. | `conditional go`: adapter execution now has a real approval gate; richer verification capture UX, final/session review ergonomics, and live manual TUI QA remain for later #143 slices. |
| 2026-05-16 | #143 stacked verification-capture slice: the TUI now stores required verification checks from live gate inputs, rejects unknown verification statuses, blocks final/session review until every required check is recorded as `passed`, sends structured verification evidence into `review_agent_session`, and blocks promotion approval until the fresh final/session review path completes. Verification: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `npm run release:check`. | `conditional go`: verification can no longer be skipped in the normal promotion path; live manual terminal QA and broader promotion ergonomics remain before #143 can close. |
| 2026-05-16 | #143 stacked verification-UX slice: the TUI now has a `verification status` command, lists required checks with current statuses, and rejects verification records whose check name does not match the required check set. Verification: `cargo test --workspace` and `npm run release:check`. | `conditional go`: the operator can discover the exact verification names and typo records no longer silently miss the gate; live manual terminal QA and promotion-path polish remain before #143 can close. |
| 2026-05-16 | Local terminal smoke on the source-built TUI binary: `target/debug/architect-mcp-tui smoke --json` returned `passed_with_warnings`, detected Codex CLI as installed and authenticated, detected the shell adapter as ready, and confirmed the gate-only run stops with `approval_required` instead of executing an adapter. | `conditional go`: source-built terminal smoke is healthy; full manual interactive TUI walkthrough and promotion-path polish remain before #143 can close. |
| 2026-05-17 | Stale local shim hardening: the npm shim now skips local TUI binaries that do not expose the current required `smoke` command, preventing an old `target/release` binary from masking a current source-built debug binary. The PTY success path also waits longer for reader output after child exit so CI does not drop short adapter output. Verification: focused shim tests, focused PTY success test, `node bin/architect-mcp-tui.cjs smoke --json`, and `npm run release:check`. | `conditional go`: source-checkout terminal smoke now works through the public npm shim path; full manual interactive TUI walkthrough and promotion-path polish remain before #143 can close. |
| 2026-05-17 | Promotion readiness slice: the TUI now exposes `promotion status`, shares promotion-readiness checks with the `promote` path, reports missing approval, isolated-worktree evidence, changed-file evidence, verification, and review gates with next actions, and blocks promotion when changed-file evidence is missing even under override. Verification: focused promotion workflow tests, `node bin/architect-mcp-tui.cjs smoke --json`, and `npm run release:check`. | `conditional go`: promotion blockers are now explicit and actionable; a full manual interactive TUI walkthrough remains before #143 can close. |
| 2026-05-17 | Scripted interactive walkthrough slice: `architect-mcp-tui walkthrough --json` now runs the command-palette engine in a throwaway git workspace from intake answers through grill, contract, plan/file review, execution approval, isolated fixture adapter run, diff inspection, verification, final/session review, `promotion status`, approval, and promotion. The slice also fixes schema-safe repo-structure review args and structured promotion-readiness parsing. Verification: `npm run tui:live-qa`, `npm run typecheck`, `npm test`, `npm run docs:build`, and `npm run release:check`. | `conditional go`: the operator flow is reproducible in CI and local terminals; visual/manual terminal rendering QA still remains before #143 can close as a full `go`. |
| 2026-05-17 | Manual 80x24 macOS terminal launch in a disposable git repo found the Inspector panel was pushed off-screen by the wide three-column layout. The TUI now switches to a narrow layout that keeps Agents, Transcript, Inspector, and Command visible; the session accepted `new app local notes QA`, tab focus changed panels, `q` exited, and `terminal-restored-ok` printed after alternate-screen teardown. Verification: focused 80x24 render test and manual TTY launch. | `conditional go`: local visual rendering now has source-checkout evidence; Linux and Windows manual terminal evidence remain separate release-readiness inputs. |
| 2026-05-17 | Adapter failure-evidence hardening: adapter timeout, crash, cancellation, non-zero exit, and truncated output are now stored as durable session issues, surfaced in the inspector, block normal promotion readiness and promotion approval, and require a successful rerun or explicit maintainer override before files can be promoted. Verification: focused run-evidence, promotion-readiness, and interactive failed-adapter tests. | `conditional go`: failed adapter runs can no longer look like clean promotion-ready success; broader real Codex adapter smoke still remains. |
| 2026-05-17 | Adapter rerun recovery hardening: after file-plan review and fresh execution approval, a rerun now replaces the managed isolated worktree for the same session/adapter, clears stale diff/review/verification/final-response/adapter-issue evidence, and applies only the new run evidence. Verification: focused session reset test, interactive fail-once-then-rerun workflow test, `npm run tui:live-qa`, and `npm run release:check`. | `conditional go`: failed runs now have a normal recovery path without needing override when the rerun succeeds; broader real Codex adapter smoke still remains. |
| 2026-05-17 | Codex adapter template hardening: the default Codex adapter now runs non-interactive `codex exec --sandbox workspace-write --color never --ephemeral`, and approved adapter execution receives the original request plus the pre-edit contract, required verification checks, and no-commit/no-promotion rules. The first real promotion-smoke attempt showed `--json` output can exceed the TUI log cap and block promotion despite a valid diff, so the default template uses plain non-interactive output. Verification: focused adapter template and gated-prompt tests. | `conditional go`: the default Codex path is now shaped for governed isolated execution; a full real Codex promotion smoke remains as release evidence. |
| 2026-05-17 | Real local Codex adapter smoke in a disposable git repo: `architect-mcp-tui run --adapter codex --jsonl --execute` used authenticated `codex-cli 0.130.0`, created only `docs/codex-adapter-smoke.md` in `.architect-mcp/worktrees/<session>/codex`, emitted `diff_evidence`, and ran `review_implementation_against_contract`, `review_repo_structure`, `review_agent_final_response`, and `review_agent_session` before ending with `review_required`. | `conditional go`: real Codex execution now has local evidence through review gates; promotion still needs a real-Codex interactive approval/promote smoke before full go. |
| 2026-05-17 | Real local Codex promotion smoke: `architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace` created a disposable git repo, confirmed Codex auth, ran the interactive gate sequence, executed Codex in an isolated worktree, refreshed implementation review after verification, passed `npm test`, approved promotion separately from execution, and promoted only `docs/codex-adapter-smoke.md`. Supporting fixes: repo-layout file hints now flow into the pre-edit contract, the adapter output display cap is 512 KiB while still blocking true truncation, and final review refreshes implementation drift review with passed verification evidence. | `conditional go`: real Codex execution-to-promotion is proven locally; broader Linux and Windows manual terminal rendering evidence remains before #143 is a full production `go`. |
| 2026-05-17 | #145 arena hardening slice: `arena run` now requires file-plan review plus explicit execution approval, requires at least two adapters, clears stale normal adapter evidence before recording candidates, and clears execution approval after the arena run. `arena select <adapter>` selects exactly one recorded candidate into the normal verification, final/session review, approval, and promotion path without copying files immediately. Fixture workflow tests cover approval blocking, isolated candidate worktrees, no auto-promotion, failed-candidate selection blocking, timeout/cancel evidence, final manual promotion of one candidate, and arena ranking evidence. Verification: `npm run tui:live-qa` and `npm run release:check`. | `conditional go`: arena execution can no longer bypass the work gate or auto-promote a winner; final PR CI and Copilot review remain before #145 is full `go`. |

## Current Slice Notes

The active implementation slice is [#145](https://github.com/tonycdr-prog/architect-mcp/issues/145). The desired outcome is a TUI arena path where multiple adapters run against the same approved contract in isolated worktrees, the TUI ranks candidates with evidence, and no candidate can be promoted without explicit selection, verification, final/session review, and promotion approval.

Current partial status: arena execution now requires file-plan review, explicit execution approval, and at least two adapters. Candidate evidence records worktree, changed files, diff stat, adapter run issues, and review gates. `arena select <adapter>` moves one successful candidate into the normal promotion path without copying files immediately, while crashed, timed-out, cancelled, or otherwise failed candidates are blocked from normal selection. Fixture workflow tests cover isolated candidates, no auto-promotion, failed-candidate blocking, ranking evidence, selected-candidate promotion, and release-gate readiness. This does not complete #145 by itself; final PR CI and Copilot review remain before a full `go`.

Before claiming #143 complete, run the TUI checks and the clean release gate:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
architect-mcp-tui smoke --json
architect-mcp-tui walkthrough --json
architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace
npm run release:check
```
