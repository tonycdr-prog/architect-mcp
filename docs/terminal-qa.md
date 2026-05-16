# Terminal QA

Use this page when testing `architect-mcp-tui` in a real terminal outside CI. The goal is to catch install, cache, checksum, shell, keyboard, mouse, resize, and JSONL problems that hosted runners often miss.

Do not paste secrets, tokens, private repository names, private file contents, or customer data into public issues.

## One-Command Smoke

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

## macOS And Linux

```bash
node --version
npm --version
npm install -g @tonycdr-prog/architect-mcp
architect-mcp-tui smoke --json > architect-mcp-tui-smoke.json
```

If the command exits non-zero, keep `architect-mcp-tui-smoke.json` if it was written and paste the terminal error into the issue.

Manual fallback commands:

```bash
architect-mcp-tui --help
architect-mcp-tui config adapters --json
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
if ($LASTEXITCODE -ne 0) { Write-Host "smoke exited with code $LASTEXITCODE" }
```

Manual fallback commands:

```powershell
architect-mcp-tui --help
architect-mcp-tui config adapters --json
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
```

Manual fallback commands:

```bash
architect-mcp-tui --help
architect-mcp-tui config adapters --json
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

For successful terminal QA, open a Terminal QA report and paste:

- OS and CPU architecture.
- Node and npm versions.
- Package version tested.
- Commands run.
- `architect-mcp-tui-smoke.json`.
- Any terminal rendering, resize, mouse, cache, checksum, or JSONL issue.

For install, checksum, cache, download, or binary launch failures, use the Install failure form.

Current public QA tracking issue: https://github.com/tonycdr-prog/architect-mcp/issues/136
