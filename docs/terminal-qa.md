# Terminal QA

Use this page when testing `architect-mcp-tui` in a real terminal outside CI. The goal is to catch install, cache, checksum, shell, keyboard, mouse, resize, and JSONL problems that hosted runners often miss.

Do not paste secrets, tokens, private repository names, private file contents, or customer data into public issues.

## One-Command Public Evidence

The preferred public evidence command on Linux and Windows is:

```bash
architect-mcp-tui terminal-evidence --markdown
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
```

It runs the smoke checks internally and writes a launch-judge-compatible summary. Use `--markdown` when pasting directly into a GitHub issue; it wraps the public-safe JSON in a fenced block that `collect-terminal-evidence` can read. Use `--json` when saving a local file for `launch-judge --terminal-evidence`. Public reports should paste this summary, not the raw smoke JSON. The command auto-detects Linux and Windows; use `--platform linux` or `--platform windows` only when a maintainer asks you to summarize already-verified external evidence.

Terminal evidence also records `environment`. Reports with `local_terminal` or `vm_or_cloud_terminal` can satisfy final manual terminal QA when the rest of the report is clean. Reports from `hosted_ci`, `container`, missing provenance, or `unknown` are still useful smoke evidence, but they remain `conditional_go` and do not replace manual Linux/Windows terminal QA. The command auto-detects hosted CI and common container markers when it can; pass `--environment vm-or-cloud-terminal` only when the run really happened in a VM or cloud dev box terminal.

Generated terminal evidence includes `collectedAt` automatically using the current UTC date. Maintainers summarizing already-verified external evidence can pass `--collected-at YYYY-MM-DD`; values outside that date format fail without echoing the raw input.

## Local Troubleshooting Smoke

The preferred smoke command is:

```bash
architect-mcp-tui smoke --json
```

It produces a secret-safe report with:

- OS, CPU, terminal, and tool versions.
- Native binary path, SHA-256, and npm shim cache location.
- `architect-mcp-tui --help` result.
- Adapter health output, including Codex auth when available.
- A live gate-only JSONL run that starts with `grill_me` and stops before adapter execution.

Missing Codex login is a warning, not an install failure. A smoke failure usually means help output, binary launch, or live MCP gate execution failed.

Keep the smoke JSON locally for troubleshooting. Do not paste the raw smoke JSON into public issues; it can include local binary/cache paths or other details that are useful for debugging but not safe public evidence.

## Hosted Published-Package Smoke

The published-package smoke workflow is hosted non-interactive baseline evidence only. It installs `@tonycdr-prog/architect-mcp@latest` into a clean temporary runner workspace on Ubuntu and Windows, points the TUI at that installed package's `dist/index.js`, then runs `architect-mcp-tui --help`, `architect-mcp-tui config adapters --json`, and a gate-only `architect-mcp-tui run --jsonl` that must stop at approval before adapter execution. This catches package install, command-surface, MCP bridge, and JSONL regressions in the published npm package, but it does not open the interactive TUI and does not replace #136 manual terminal QA.

## Scripted Walkthrough

Run the guarded command-palette flow in a throwaway git workspace:

```bash
architect-mcp-tui walkthrough --json > architect-mcp-tui-walkthrough.json
```

The report should end with `status` set to `passed`, `finalApproval` set to `promoted`, and at least one promoted file. This command uses the same interactive workflow engine as the TUI, including grill, contract, plan review, file-plan review, adapter approval, isolated adapter execution, diff inspection, verification, final/session review, promotion status, and promotion. It is still not a substitute for visually opening the TUI and checking terminal rendering.

## Real Adapter Promotion Smoke

When Codex is installed and authenticated, run the local promotion smoke in a throwaway repo:

```bash
architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace > architect-mcp-tui-promotion-smoke.json
```

This command creates a disposable git workspace, runs the selected adapter only after the TUI execution approval gate, verifies the isolated worktree, runs final/session review, then requires a separate promotion approval before copying files back. The report should end with `status` set to `passed`, `finalApproval` set to `promoted`, and exactly one promoted file: `docs/codex-adapter-smoke.md`. Keep the workspace path from the report when investigating failures.

## macOS And Linux

```bash
node --version
npm --version
npm install -g @tonycdr-prog/architect-mcp
architect-mcp-tui smoke --json > architect-mcp-tui-smoke.json
# Linux only:
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
```

If the command exits non-zero, keep `architect-mcp-tui-smoke.json` locally if it was written and paste only the shortest useful terminal error into the issue.

Manual fallback commands:

```bash
architect-mcp-tui --help
architect-mcp-tui config adapters --json
architect-mcp-tui walkthrough --json
architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace
architect-mcp-tui run \
  --prompt "Build a tiny local notes app for one developer. Flows: create, edit, delete, and search notes. Stack: TypeScript CLI. Risks: file corruption and unclear persistence. Verification: unit tests for CRUD and search plus npm test." \
  --adapter codex \
  --jsonl
```

## Windows PowerShell

```powershell
node --version
npm --version
npm install -g @tonycdr-prog/architect-mcp
architect-mcp-tui smoke --json > architect-mcp-tui-smoke.json
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
if ($LASTEXITCODE -ne 0) { Write-Host "smoke exited with code $LASTEXITCODE" }
```

Manual fallback commands:

```powershell
architect-mcp-tui --help
architect-mcp-tui config adapters --json
architect-mcp-tui walkthrough --json
architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace
architect-mcp-tui run `
  --prompt "Build a tiny local notes app for one developer. Flows: create, edit, delete, and search notes. Stack: TypeScript CLI. Risks: file corruption and unclear persistence. Verification: unit tests for CRUD and search plus npm test." `
  --adapter codex `
  --jsonl
```

## Windows Git Bash

```bash
node --version
npm --version
npm install -g @tonycdr-prog/architect-mcp
architect-mcp-tui smoke --json > architect-mcp-tui-smoke.json
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
```

Manual fallback commands:

```bash
architect-mcp-tui --help
architect-mcp-tui config adapters --json
architect-mcp-tui walkthrough --json
architect-mcp-tui promotion-smoke --adapter codex --json --keep-workspace
architect-mcp-tui run \
  --prompt "Build a tiny local notes app for one developer. Flows: create, edit, delete, and search notes. Stack: TypeScript CLI. Risks: file corruption and unclear persistence. Verification: unit tests for CRUD and search plus npm test." \
  --adapter codex \
  --jsonl
```

## Optional Interactive Check

Run the TUI in a disposable directory:

```bash
mkdir architect-mcp-tui-qa
cd architect-mcp-tui-qa
git init
architect-mcp-tui
```

Check:

- The screen opens without corruption.
- Keyboard input appears in the command area.
- Mouse scroll, click, and resize do not corrupt the screen.
- Exiting leaves the terminal usable.

Do not run adapter execution in a public repository unless you are intentionally testing a disposable clone.

## Report Results

For successful Linux or Windows terminal QA, open a Terminal QA report and paste the public-safe issue-comment output from `architect-mcp-tui terminal-evidence --markdown`. Maintainers should be able to collect each platform report and run:

```bash
architect-mcp-tui launch-judge --json --terminal-evidence terminal-evidence.json
architect-mcp-tui launch-judge --json --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui launch-judge --public-summary --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --public-summary --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136
architect-mcp-tui launch-readiness --json --repo tonycdr-prog/architect-mcp --stack-from-pr 190 --blocker 136 --terminal-evidence-issue 136 --waive-blocker 136="maintainer accepted launch with platform QA waiver" --waive-terminal-evidence 136="maintainer accepted launch without manual Linux/Windows terminal evidence"
```

Use the repeated flag form when Linux and Windows evidence arrives as separate files. Use `collect-terminal-evidence` when Linux and Windows evidence arrives as fenced JSON in issue comments generated by `terminal-evidence --markdown`. If no blocks are found, the collector asks testers to post output from `architect-mcp-tui terminal-evidence --markdown`. The launch judge merges the reports and validates the combined evidence without requiring hand-edited JSON. Maintainers should use `launch-judge --public-summary` or `launch-readiness --public-summary` when posting the decision back to an issue or release note; the summary keeps the decision and stack/evidence status while omitting freeform evidence text, full PR/check payloads, raw report internals, and local paths.

When reports are posted as fenced JSON in a GitHub issue, maintainers can collect and validate them without hand-copying each comment:

```bash
architect-mcp-tui collect-terminal-evidence --json --repo tonycdr-prog/architect-mcp --issue 136
```

This command reads issue comments through GitHub CLI, extracts public-safe terminal-evidence JSON blocks, validates them with the same launch-judge safety rules, and prints merged evidence that can be saved and passed to `launch-judge --terminal-evidence`. It is read-only and does not create, edit, or close issues.

`launch-readiness` is the maintainer rollup for the release stack plus the terminal-evidence issue. It is also read-only, can discover the base-to-head chain for a stacked release with `--stack-from-pr`, accepts repeated `--required-check <name>` inputs that must exist on every listed PR, reports `go`, `conditional_go`, or `no_go`, and keeps waivers separate from real Linux/Windows terminal evidence. Hosted CI terminal-evidence summaries are baseline signals only and do not satisfy issue #136 manual Windows/Linux terminal QA. If maintainers intentionally launch without real manual Linux/Windows evidence, use `--waive-terminal-evidence ISSUE=reason`; unsafe or malformed evidence remains `no_go` even with a waiver.

The generated evidence has this shape:

```json
{
  "schemaVersion": 1,
  "reports": [
    {
      "platform": "linux",
      "status": "passed",
      "environment": "local_terminal",
      "source": "issue #136 public-safe terminal QA report",
      "commandSummary": "architect-mcp-tui --help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "interactive launch opened cleanly; keyboard, mouse scroll, resize, and exit behaved as expected"
    }
  ]
}
```

Use `platform` as `linux` or `windows`, `status` as `passed`, `passed_with_warnings`, or `failed`, and `environment` as `local_terminal`, `vm_or_cloud_terminal`, `container`, `hosted_ci`, or `unknown`. For each platform, also include:

- OS and CPU architecture.
- Node and npm versions.
- Package version tested.
- Commands run.
- Adapter health summary.
- Whether `architect-mcp-tui smoke --json`, `walkthrough --json`, or `promotion-smoke --json` was run.
- Any terminal rendering, resize, mouse, cache, checksum, or JSONL issue.

Do not submit unchanged template values such as `REPLACE with ...` or `issue #136 public-safe terminal QA report`. The launch judge treats those as placeholder evidence and keeps the result at `conditional_go` until real platform-specific evidence is supplied.

Keep raw `architect-mcp-tui-smoke.json`, `architect-mcp-tui-walkthrough.json`, and `architect-mcp-tui-promotion-smoke.json` files local unless a maintainer asks for a redacted excerpt. Public issues should contain summaries, not raw logs, absolute local paths, cache paths, private repo names, tokens, or full environment dumps.

For install, checksum, cache, download, or binary launch failures, use the Install failure form.

Current public QA tracking issue: https://github.com/tonycdr-prog/architect-mcp/issues/136
