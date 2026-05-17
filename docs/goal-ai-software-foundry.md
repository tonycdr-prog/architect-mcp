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
| TUI production control loop | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | Review-ready stack; not merged | User can complete a TUI work-gate path to verified promotion without bypassing gates. |
| Adapter evidence and promotion hard gates | [#144](https://github.com/tonycdr-prog/architect-mcp/issues/144) | Review-ready stack; not merged | Adapter runs record evidence, failures cannot look successful, and promotion is review-gated. |
| Multi-agent arena hardening | [#145](https://github.com/tonycdr-prog/architect-mcp/issues/145) | Review-ready stack; not merged | Multiple candidates run in isolated worktrees and are ranked without auto-promotion. |
| MCP catalog and install-plan flow | [#146](https://github.com/tonycdr-prog/architect-mcp/issues/146) | Review-ready stack; not merged | Recommendations require clarified need, dry-run install plans, security review, and approval before config writes. |
| New app to private repo foundry path | [#147](https://github.com/tonycdr-prog/architect-mcp/issues/147) | Review-ready stack; not merged | A clarified app idea can become a private repo with CI, docs, agent instructions, env template, and first PR evidence. |
| Governance audit and drift evidence loop | [#148](https://github.com/tonycdr-prog/architect-mcp/issues/148) | Review-ready stack; not merged | Maintained repos can be audited read-only for drift, stale docs, weak tests, unsafe config, and memory safety. |

Future hosted and team mode should stay behind the local-first proof. Hosted work is not launch-blocking for the local operator path, and hosted mode must keep local-only tools excluded.

## Open Stack Snapshot

This snapshot records active implementation evidence only. Do not treat any unmerged PR in this table as released product behavior.

| PR | Issue | Slice | Latest judge state | Remaining gate |
| --- | --- | --- | --- | --- |
| [#150](https://github.com/tonycdr-prog/architect-mcp/pull/150) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | TUI intake answers shape live gate inputs. | `conditional go` | Stack landing and broader terminal evidence. |
| [#151](https://github.com/tonycdr-prog/architect-mcp/pull/151) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | Adapter execution requires explicit approval. | `conditional go` | Stack landing and promotion-path proof. |
| [#152](https://github.com/tonycdr-prog/architect-mcp/pull/152) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | Verification evidence blocks final/session review and promotion. | `conditional go` | Stack landing and terminal QA. |
| [#153](https://github.com/tonycdr-prog/architect-mcp/pull/153) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | Verification status UX and typo-safe records. | `conditional go` | Stack landing and terminal QA. |
| [#154](https://github.com/tonycdr-prog/architect-mcp/pull/154) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) | Shim smoke skips stale local TUI binaries. | `conditional go` | Stack landing. |
| [#155](https://github.com/tonycdr-prog/architect-mcp/pull/155) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143), [#144](https://github.com/tonycdr-prog/architect-mcp/issues/144) | Promotion readiness blockers are visible and shared with promotion. | `conditional go` | Stack landing and full manual QA. |
| [#156](https://github.com/tonycdr-prog/architect-mcp/pull/156) | [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143), [#144](https://github.com/tonycdr-prog/architect-mcp/issues/144) | Scripted walkthrough, real Codex promotion smoke, adapter failure recovery. | `conditional go` | Broader manual Linux and Windows terminal rendering evidence. |
| [#157](https://github.com/tonycdr-prog/architect-mcp/pull/157) | [#145](https://github.com/tonycdr-prog/architect-mcp/issues/145) | Arena candidate isolation, ranking, and manual selection. | `conditional go` | Stack landing in order. |
| [#158](https://github.com/tonycdr-prog/architect-mcp/pull/158) | [#146](https://github.com/tonycdr-prog/architect-mcp/issues/146) | Guarded MCP recommendations, install plans, review, approval, and write boundary. | `conditional go` | Stack landing and foundry integration follow-through. |
| [#159](https://github.com/tonycdr-prog/architect-mcp/pull/159) | [#147](https://github.com/tonycdr-prog/architect-mcp/issues/147) | Private-by-default repo foundry plan and approval boundary. | `conditional go` | Live creation path. |
| [#160](https://github.com/tonycdr-prog/architect-mcp/pull/160) | [#147](https://github.com/tonycdr-prog/architect-mcp/issues/147) | Staged repo execution boundary before private GitHub creation. | `conditional go` | Live private-repo smoke. |
| [#161](https://github.com/tonycdr-prog/architect-mcp/pull/161) | [#147](https://github.com/tonycdr-prog/architect-mcp/issues/147) | Dry-run/live foundry smoke with private repo, CI, and draft PR proof. | `conditional go` | Stack landing and retained proof repo decision. |
| [#162](https://github.com/tonycdr-prog/architect-mcp/pull/162) | [#148](https://github.com/tonycdr-prog/architect-mcp/issues/148) | Read-only governance audit, recurring workflow, public-safe reports, fresh public-repo samples. | `conditional go` | Stack landing and post-merge audit confirmation. |

Supporting launch slices:

| PR | Issue | Slice | Latest judge state | Remaining gate |
| --- | --- | --- | --- | --- |
| [#163](https://github.com/tonycdr-prog/architect-mcp/pull/163) | [#137](https://github.com/tonycdr-prog/architect-mcp/issues/137) | npm publish environment hardening and trusted-publisher migration docs. | `conditional go` | Maintainer-side npm trusted publisher setup and token removal. |
| [#164](https://github.com/tonycdr-prog/architect-mcp/pull/164) | [#136](https://github.com/tonycdr-prog/architect-mcp/issues/136) | Terminal QA docs aligned with published `v0.2.1` TUI command surface. | `conditional go` | Real Windows and Linux human terminal evidence. |

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
| 2026-05-17 | [#150](https://github.com/tonycdr-prog/architect-mcp/pull/150) through [#156](https://github.com/tonycdr-prog/architect-mcp/pull/156) built the TUI control-loop stack: intake answers, execution approval, verification gates, promotion readiness, scripted walkthrough, adapter failure recovery, and real Codex promotion smoke. | `conditional go`; implementation evidence is strong, but broader manual Linux and Windows terminal rendering evidence remains. |
| 2026-05-17 | [#157](https://github.com/tonycdr-prog/architect-mcp/pull/157) added arena isolation, ranking, failed-candidate blocking, and manual selection. | `conditional go`; stack must land in order before issue closure. |
| 2026-05-17 | [#158](https://github.com/tonycdr-prog/architect-mcp/pull/158) added guarded MCP recommendation, dry-run install planning, install review, explicit approval, and config-write boundaries. | `conditional go`; foundry integration remains a follow-through concern. |
| 2026-05-17 | [#159](https://github.com/tonycdr-prog/architect-mcp/pull/159) through [#161](https://github.com/tonycdr-prog/architect-mcp/pull/161) added the private-repo foundry plan, staging boundary, and live private-repo smoke with CI and draft PR evidence. | `conditional go`; stack must land and retained proof repo decision must be accepted. |
| 2026-05-17 | [#162](https://github.com/tonycdr-prog/architect-mcp/pull/162) added read-only governance audit, recurring workflow, public-safe report template, and fresh public-repo samples. | `conditional go`; post-merge audit confirmation remains. |
| 2026-05-17 | [#163](https://github.com/tonycdr-prog/architect-mcp/pull/163) and [#164](https://github.com/tonycdr-prog/architect-mcp/pull/164) hardened launch support surfaces for npm publishing and terminal QA evidence. | `conditional go`; maintainer-side publishing setup and Windows/Linux terminal reports remain external gates. |

## Current Slice Notes

The active work is stack landing and external evidence collection, not redefining the goal downward. The open PR stack has review-ready implementation slices across [#143](https://github.com/tonycdr-prog/architect-mcp/issues/143) through [#148](https://github.com/tonycdr-prog/architect-mcp/issues/148), but those slices are not merged or released until they land on `main` in order and their remaining evidence gates are satisfied.

Do not close the epic or mark the runtime `/goal` complete until the current-state evidence proves the full objective:

- TUI control loop merged, published, and manually smoke-tested beyond hosted CI.
- Adapter execution and promotion paths approval-gated in real use.
- Arena, MCP install, foundry, and governance flows merged and documented without overclaiming.
- Public launch docs match the package actually available to users.
- `npm run release:check` passes from the final clean checkout.

Before claiming a release-sensitive slice complete, run the TUI checks and the clean release gate:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
architect-mcp-tui smoke --json
npm run tui:live-qa
npm run release:check
```
