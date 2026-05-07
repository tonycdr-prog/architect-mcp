# Security Policy

## Supported Versions

Security fixes target the latest published version and the default branch.

## Reporting A Vulnerability

Please report suspected vulnerabilities privately by opening a GitHub security advisory for this repository. Do not file public issues for secrets, exploit details, or private configuration data.

Include:

- affected version or commit
- reproduction steps
- impact and affected MCP tools or transports
- whether credentials, local files, or remote services are involved

## Project Security Expectations

- Do not commit real secrets. Use `.env.example` with safe placeholders.
- Pin executable MCP server dependencies to exact versions.
- Keep hosted HTTP mode stateless unless a future product layer explicitly owns persistence.
- Treat local workspace scanning and user MCP config scanning as local-only tools.

