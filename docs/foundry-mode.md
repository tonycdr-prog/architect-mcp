# Foundry Mode

Foundry Mode is architect-mcp's repo-native actionability loop. It turns repo signals, review findings, verification evidence, and maintainer expectations into preview decisions that a human can approve, reject, or route elsewhere.

The current implementation is local-first and preview-only. It does not create branches, commits, issues, pull requests, comments, labels, releases, persistent ledgers, or public disclosures without a separate approved workflow outside the MCP tool response.

## Current Shipped Behavior

The advanced MCP surface currently provides these Foundry tools:

- `derive_repo_constitution`: derives repo instructions, PR templates, recent accepted PR style, CI, release policy, labels, package metadata, language hints, maintainer constraints, and findings from caller-supplied public-safe evidence.
- `derive_local_repo_constitution`: performs the same constitution derivation from a trusted local checkout and returns public-safe summaries rather than raw artifact bodies.
- `normalize_foundry_evidence`: converts review findings, external-tool summaries, verification summaries, coverage data, suppression candidates, and repo constitution output into stable public-safe evidence records.
- `score_foundry_actionability`: scores normalized evidence for PR-preview suitability, human review, exception, or no-op handling.
- `route_foundry_decisions`: routes scored findings to `pr_preview`, `architect_issue`, `exception`, `no_op`, or `ask_human` ledger entries with public-safe evidence aliases and mutation policy.
- `forge_foundry_previews`: renders preview-only pull request, architect-issue, exception, no-op, and human-question artifacts from ledger entries.
- `run_foundry_eval_corpus`: runs deterministic offline large/small/tiny repo fixtures that prove route coverage, public-safety redaction, signal-quality regressions, and zero server writes without network access.

The TUI currently exposes `architect-mcp-tui foundry-audit`. That command chains the local MCP tools through a read-only audit and renders route, score, evidence count, redaction risk, approval state, next action, preview counts, and mutation boundary. `--public-summary` omits local paths, raw command output markers, raw MCP payloads, raw repo content, token-shaped values, and private diagnostics.

`architect-mcp-tui foundry-smoke` is a separate repo-foundry proof path. Its dry-run mode does not create a repository. Its live mode requires both `--execute` and `--confirm-private-repo-mutation`, and it remains governed by the explicit private proof-repo approval and retention rules documented in the Rust TUI guide.

## Future Planned Behavior

The roadmap can extend Foundry Mode into persistent operator workflows, but those behaviors are not current MCP mutations:

- persistent public-safe ledger storage after an approved client or host chooses a storage location
- approved GitHub issue, pull request, comment, label, or branch creation using explicit host-side approval gates
- recurring read-only audit schedules that create public-safe issue proposals only after review
- richer patch preview generation with verification receipts and maintainer-style reconciliation
- team/hosted dashboards that keep local-only workspace scanning excluded from hosted-safe tool surfaces

Future planned behavior must keep the same rule: proposal is not approval. A preview can recommend a PR, issue, exception, no-op, or human question, but the mutation boundary stays outside the MCP report until the maintainer approves a concrete action.

## Approval Gates

Foundry routes have different approval states:

- `pr_preview`: approval-required preview. A maintainer must approve the target repo, branch, diff, verification plan, PR body, and disclosure boundary before any PR is opened.
- `architect_issue`: approval-required preview. A maintainer must approve public wording, issue target, labels, and evidence disclosure before any issue is opened.
- `exception`: approval-required preview. A maintainer must approve the risk acceptance, expiry, owner, and verification evidence before an exception is recorded as policy.
- `ask_human`: blocking question. The agent should not convert it into mutation until the maintainer resolves the ambiguity, disclosure risk, or missing verification.
- `no_op`: no mutation. The finding is recorded as not worth acting on unless new evidence changes the decision.

No autonomous external mutation is part of Foundry Mode V1. Direct MCP clients, TUI clients, and hosted adapters must treat Foundry output as evidence and drafts, not as permission to write to GitHub or a local checkout.

## Public-Safety Rules

Public Foundry output must be shareable in issues, PRs, docs, and release notes without exposing private or local evidence. Public summaries and previews must avoid:

- raw repo content or full file excerpts
- raw MCP payloads, raw external-tool payloads, and command transcripts
- local checkout paths, home directories, temp paths, and private repo paths
- token-shaped values, secrets, private URLs, and private target names
- sensitive security details before disclosure is explicitly approved

When a finding cannot be made public safely, the route should become `ask_human` or an internal-only exception candidate until disclosure is approved.

## PR And Issue Preview Contract

`forge_foundry_previews` creates draft text only. The preview footer must remain:

```text
Generated by architect-mcp Foundry Mode as an approval-required preview: https://github.com/tonycdr-prog/architect-mcp
```

Pull request previews reconcile repository style in this order:

1. Repository PR templates are hard maintainer signals.
2. Recent accepted PR headings are advisory fallback when templates are missing, hidden-comment-only, sparse, or plausibly stale.
3. Recent PR style never overrides explicit repo instructions, security policy, required verification, or PR-template requirements.
4. When template and recent accepted style diverge, the preview should preserve template headings first and warn that recent style is advisory.

Architect issue previews, exception records, no-op records, and human questions use ledger-local evidence aliases and public-safe rationale. They are not GitHub writes.

## Signal-Quality Inputs

Foundry Mode V1 depends on the audit-signal hardening issues created before the #320 implementation track:

| Issue | Signal quality role |
| --- | --- |
| [#310](https://github.com/tonycdr-prog/architect-mcp/issues/310) | Distinguish docs examples and client/server warning context before scoring findings. |
| [#311](https://github.com/tonycdr-prog/architect-mcp/issues/311) | Make large-repo scan coverage, truncation, and capped detailed findings visible. |
| [#312](https://github.com/tonycdr-prog/architect-mcp/issues/312) | Preserve TSX and environment-access context so framework-specific patterns are not overflagged. |
| [#313](https://github.com/tonycdr-prog/architect-mcp/issues/313) | Suppress generated, vendored, fixture, and oversized data noise before actionability scoring. |
| [#314](https://github.com/tonycdr-prog/architect-mcp/issues/314) | Use repo PR templates and recent accepted PR style as maintainer-fit signals. |
| [#315](https://github.com/tonycdr-prog/architect-mcp/issues/315) | Recognize conventional entrypoints so standard app roots do not become low-value PRs. |
| [#316](https://github.com/tonycdr-prog/architect-mcp/issues/316) | Record environment-scatter evidence with enough context to separate real drift from expected config. |
| [#317](https://github.com/tonycdr-prog/architect-mcp/issues/317) | Keep documentation-intelligence findings scoped to actionable repo behavior. |
| [#318](https://github.com/tonycdr-prog/architect-mcp/issues/318) | Treat missing `AGENTS.md` in external repos as a hard gate only when the repo context makes that legitimate. |
| [#319](https://github.com/tonycdr-prog/architect-mcp/issues/319) | Add Python, Rust, Go, and framework-profile context before deriving maintainer-facing findings. |

## Staged Roadmap

| Slice | Status |
| --- | --- |
| [#321](https://github.com/tonycdr-prog/architect-mcp/issues/321) Actionability scoring | Landed in PR #334. |
| [#322](https://github.com/tonycdr-prog/architect-mcp/issues/322) Decision-ledger routing | Landed in PR #335. |
| [#323](https://github.com/tonycdr-prog/architect-mcp/issues/323) Repo constitution | Landed in PR #332. |
| [#324](https://github.com/tonycdr-prog/architect-mcp/issues/324) Evidence normalization | Landed in PR #333. |
| [#325](https://github.com/tonycdr-prog/architect-mcp/issues/325) TUI read-only audit view | Landed in PR #337. |
| [#326](https://github.com/tonycdr-prog/architect-mcp/issues/326) PR/issue forge previews | Landed in PR #336. |
| [#327](https://github.com/tonycdr-prog/architect-mcp/issues/327) Offline eval corpus | Landed in PR #338. |
| [#328](https://github.com/tonycdr-prog/architect-mcp/issues/328) Operating contract docs | Active documentation slice. |

## Recommended Operator Loop

1. Run the work gate for the change being considered.
2. Derive the repo constitution from trusted local evidence or supplied public-safe evidence.
3. Normalize audit findings and verification summaries.
4. Score actionability before deciding whether a maintainer-facing change is worth proposing.
5. Route each scored item to a PR preview, architect issue, exception, no-op, or human question.
6. Forge previews only after the route is explicit.
7. Review the public-safe preview, verification plan, and mutation boundary.
8. Approve or reject a separate GitHub or local mutation workflow.
9. Record only public-safe evidence in the goal ledger, PR body, issue, or release note.
