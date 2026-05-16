# Read-Only Smoke Matrix

Use this matrix after releases and tuning PRs to test architect-mcp on fresh public repositories without posting issues, opening pull requests, or changing those repositories.

## Rules

- Clone or inspect public repos read-only.
- Do not create issues, pull requests, comments, discussions, stars, forks, or branches on third-party repos.
- Record the exact repo URL, commit SHA, date, tool surface, review mode, and command or client path.
- Summarize findings in the architect-mcp repo or chat only.
- Treat false positives as tuning candidates, not as criticism of the target repo.

## Coverage Targets

| Ecosystem | Minimum Sample | Review Mode | What To Watch |
| --- | --- | --- | --- |
| TypeScript app | 2 repos | `audit` | Generated assets, nested barrels, public media, lockfiles |
| TypeScript tooling or library | 1 repo | `audit` | CLI entrypoints, package metadata, declaration files |
| Python app or library | 1 repo | `audit` | Framework app files, notebooks, generated docs |
| Go service or CLI | 1 repo | `audit` | Root commands, generated clients, vendored output |
| Rust service or CLI | 1 repo | `audit` | `src/main.rs`, generated bindings, examples |
| Java service | 1 repo | `audit` | Verbose framework classes, generated sources, build files |

## Report Template

```markdown
## Repo
- URL:
- Commit:
- Date:
- Ecosystem:
- Tool surface:
- Review mode:

## Result
- Status:
- High-signal findings:
- Noise findings:
- Missed findings:
- Suggested tuning:

## Evidence
- Commands or MCP calls:
- Notes:
```

## Launch Gate

For a release candidate, run at least one fresh sample per ecosystem and keep `npm run release:check` as the final clean-checkout release gate. The smoke matrix informs tuning, but it does not replace the deterministic release gate.
