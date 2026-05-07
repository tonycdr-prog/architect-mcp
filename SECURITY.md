# Security Policy

## Supported Versions

Security fixes are handled on the current `main` branch until a formal release line exists.

## Reporting A Vulnerability

Please do not report vulnerabilities, secrets, exploit details, or private repository data in public issues or pull request comments.

Report security concerns privately through GitHub's private vulnerability reporting flow if it is enabled for the repository, or contact the maintainer directly with:

- the affected version or commit
- the impacted surface, such as MCP stdio, HTTP transport, local workspace scanning, generated artifacts, or future hosted product code
- enough detail to reproduce the issue without including real secrets
- whether the issue appears exploitable locally, remotely, or only through a trusted client

Expected triage flow:

1. The maintainer acknowledges the report.
2. The issue is reproduced or scoped.
3. A fix is prepared privately when disclosure risk is material.
4. The public issue or advisory is created only after sensitive details are removed.

## Security Expectations

- Never include live credentials, tokens, customer data, private source code, or exploit payloads in reports unless explicitly requested through a private channel.
- Prefer minimal proof-of-concept inputs over destructive commands.
- For dependency or supply-chain findings, include package names, versions, and advisory links where available.
