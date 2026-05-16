# Obsidian Project Memory

Obsidian can be useful as a human-facing knowledge base, but architect-mcp should not depend on an Obsidian MCP server for launch readiness. The default project memory surface remains repo-local Markdown plus optional reviewed MCP memory proposals.

## Default Shape

Use ordinary Markdown files first:

- `docs/architecture-contract.md`: current architectural contract.
- `docs/build-plan.md`: implementation slices and verification.
- `AGENTS.md`: agent rules, repo boundaries, and proof expectations.
- `docs/decisions/*.md`: durable architecture decisions when needed.
- `docs/read-only-smoke-matrix.md`: post-launch smoke evidence.

This keeps project memory inspectable in Git, reviewable in pull requests, and available to every coding agent without a private account dependency.

## Obsidian Role

Obsidian is appropriate for maintainer notes, research clippings, and cross-project thinking when the maintainer wants a personal vault. It should not become a required runtime dependency or a hidden source of project truth.

Use Obsidian only when:

- The content is useful to a human maintainer outside the repo.
- The same decision or rule is mirrored into repo docs before it affects code.
- No secrets, customer data, private repo content, or credential material is stored.

## Memory MCP Boundary

The Memory MCP server can help with session continuity, but it must remain reviewed and opt-in:

- Propose memories with `extract_harness_memory`.
- Check them with `review_memory_relevance`.
- Apply only non-secret, relevant, non-sensitive entries.
- Keep source-of-truth project rules in repo files.

## No-Go Content

Never store:

- API keys, tokens, passwords, recovery codes, or private keys.
- Customer data, support tickets, private emails, or production database excerpts.
- Vulnerability details that should follow `SECURITY.md`.
- Unverified claims presented as root cause.
- Repo instructions that have not been mirrored into `AGENTS.md` or docs.

## Decision

Do not add Obsidian MCP as a default install recommendation yet. Add it later only if repeated real workflow evidence shows that repo-local Markdown plus optional Memory MCP review is not enough.
