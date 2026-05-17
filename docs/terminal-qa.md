# Terminal QA

Use this page when testing `architect-mcp-tui` in a real terminal outside CI. The goal is to catch install, cache, checksum, shell, keyboard, mouse, resize, and JSONL problems that hosted runners often miss.

Do not paste secrets, tokens, private repository names, private file contents, or customer data into public issues.

## One-Command Public Evidence

The preferred public evidence command on Linux and Windows is:

```bash
architect-mcp-tui terminal-evidence --json > terminal-evidence.json
```

It runs the smoke checks internally and writes a launch-judge-compatible summary. Public reports should paste this summary, not the raw smoke JSON. The command auto-detects Linux and Windows; use `--platform linux` or `--platform windows` only when a maintainer asks you to summarize already-verified external evidence.

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

For successful Linux or Windows terminal QA, open a Terminal QA report and paste the public-safe output from `architect-mcp-tui terminal-evidence --json`. Maintainers should be able to save each platform report and run:

```bash
architect-mcp-tui launch-judge --json --terminal-evidence terminal-evidence.json
architect-mcp-tui launch-judge --json --terminal-evidence linux-evidence.json --terminal-evidence windows-evidence.json
```

Use the repeated flag form when Linux and Windows evidence arrives as separate issue comments or files. The launch judge merges the reports and validates the combined evidence without requiring hand-edited JSON.

The generated evidence has this shape:

```json
{
  "schemaVersion": 1,
  "reports": [
    {
      "platform": "linux",
      "status": "passed",
      "source": "issue #136 public-safe terminal QA report",
      "commandSummary": "architect-mcp-tui --help, config adapters --json, and gate-only run --jsonl passed",
      "collectedAt": "2026-05-17",
      "notes": "interactive launch opened cleanly; keyboard, mouse scroll, resize, and exit behaved as expected"
    }
  ]
}
```

Use `platform` as `linux` or `windows`, and `status` as `passed`, `passed_with_warnings`, or `failed`. For each platform, also include:

- OS and CPU architecture.
- Node and npm versions.
- Package version tested.
- Commands run.
- Adapter health summary.
- Whether `architect-mcp-tui smoke --json`, `walkthrough --json`, or `promotion-smoke --json` was run.
- Any terminal rendering, resize, mouse, cache, checksum, or JSONL issue.

Keep raw `architect-mcp-tui-smoke.json`, `architect-mcp-tui-walkthrough.json`, and `architect-mcp-tui-promotion-smoke.json` files local unless a maintainer asks for a redacted excerpt. Public issues should contain summaries, not raw logs, absolute local paths, cache paths, private repo names, tokens, or full environment dumps.

For install, checksum, cache, download, or binary launch failures, use the Install failure form.

Current public QA tracking issue: https://github.com/tonycdr-prog/architect-mcp/issues/136
