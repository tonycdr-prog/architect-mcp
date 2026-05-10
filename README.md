# architect-mcp

<p align="center">
  <a href="https://github.com/tonycdr-prog/architect-mcp/releases/latest"><img alt="latest release" src="https://img.shields.io/github/v/release/tonycdr-prog/architect-mcp?style=for-the-badge&label=release"></a>
  <a href="https://github.com/tonycdr-prog/architect-mcp/actions/workflows/ci.yml"><img alt="CI status" src="https://img.shields.io/github/actions/workflow/status/tonycdr-prog/architect-mcp/ci.yml?branch=main&style=for-the-badge&label=ci"></a>
  <a href="https://github.com/tonycdr-prog/architect-mcp/actions/workflows/pages.yml"><img alt="Docs status" src="https://img.shields.io/github/actions/workflow/status/tonycdr-prog/architect-mcp/pages.yml?branch=main&style=for-the-badge&label=docs"></a>
  <a href="https://github.com/tonycdr-prog/architect-mcp/blob/main/LICENSE"><img alt="MIT license" src="https://img.shields.io/github/license/tonycdr-prog/architect-mcp?style=for-the-badge"></a>
  <img alt="Node 20+" src="https://img.shields.io/badge/node-%3E%3D20-339933?style=for-the-badge">
</p>

<p align="center">
  <img alt="MCP server" src="https://img.shields.io/badge/MCP-server-111827?style=flat-square">
  <img alt="local first" src="https://img.shields.io/badge/local--first-default-2563eb?style=flat-square">
  <img alt="agent work gate" src="https://img.shields.io/badge/agent%20work%20gate-core-7c3aed?style=flat-square">
  <img alt="hosted safe" src="https://img.shields.io/badge/hosted--safe-stateless-059669?style=flat-square">
  <img alt="verification required" src="https://img.shields.io/badge/verification-required-d97706?style=flat-square">
</p>

architect-mcp keeps coding agents honest. It is a local-first agent work gate that clarifies intent before edits, constrains the plan, reviews implementation drift, and requires verification evidence before completion.

Full docs: [tonycdr-prog.github.io/architect-mcp](https://tonycdr-prog.github.io/architect-mcp/)

## Launch Snapshot

| Surface | What It Gates |
| --- | --- |
| Core work gate | `grill_me`, pre-edit contracts, plan review, drift review, final/session honesty checks |
| Advanced maturity | Stack packs, standards intelligence, governance evals, operating-model evals |
| Hosted mode | Stateless `/mcp`, explicit file summaries, no local workspace scanning |
| Release gate | `npm run release:check` |

## Quick Start

The npm registry name `architect-mcp` is owned by a different package. This project keeps the `architect-mcp` command name, but package distribution uses the scoped package identity `@tonycdr-prog/architect-mcp` and GitHub release tarballs until npm publishing is configured for that scope.

```bash
npm install -g https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.0/tonycdr-prog-architect-mcp-0.1.0.tgz
architect-mcp
```

For a source checkout:

```bash
npm install
npm run build
node dist/index.js
```

For local MCP clients:

```json
{
  "mcpServers": {
    "architect-mcp": {
      "command": "npx",
      "args": [
        "-y",
        "--package",
        "https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.0/tonycdr-prog-architect-mcp-0.1.0.tgz",
        "architect-mcp"
      ]
    }
  }
}
```

## Core Flow

1. Run `grill_me` until blockers are gone.
2. Create a pre-edit contract with `create_pre_edit_contract`.
3. Review the build plan with `review_build_plan`.
4. Review proposed files with `review_proposed_file_plan`.
5. Review implementation with `review_repo_structure` and `review_implementation_against_contract`.
6. Review final output with `review_agent_final_response` or `review_agent_session`.

The default MCP surface exposes exactly these eight work-gate tools:

- `grill_me`
- `create_pre_edit_contract`
- `review_build_plan`
- `review_proposed_file_plan`
- `review_repo_structure`
- `review_implementation_against_contract`
- `review_agent_final_response`
- `review_agent_session`

Use the advanced surface for stack packs, standards intelligence, governance evals, operating-model evals, productization boundary evals, repo-quality gates, and local workspace utilities:

```bash
ARCHITECT_MCP_TOOL_SURFACE=advanced architect-mcp
```

Historical V3-V10 labels remain in tool names, scripts, tests, and document filenames for compatibility. Public documentation frames them as advanced maturity criteria rather than public product versions.

## Docs

- [Getting Started](https://tonycdr-prog.github.io/architect-mcp/getting-started)
- [MCP Client Setup](https://tonycdr-prog.github.io/architect-mcp/mcp-client-setup)
- [Core Work Gate](https://tonycdr-prog.github.io/architect-mcp/core-work-gate)
- [Tool Reference](https://tonycdr-prog.github.io/architect-mcp/tool-reference)
- [Hosted Mode](https://tonycdr-prog.github.io/architect-mcp/hosted-mode)
- [Stack Packs](https://tonycdr-prog.github.io/architect-mcp/stack-packs)
- [Release Readiness](https://tonycdr-prog.github.io/architect-mcp/release-readiness)
- [Compatibility And Advanced Maturity Criteria](https://tonycdr-prog.github.io/architect-mcp/compatibility)

## Release Readiness

Use the clean-checkout release gate before release-sensitive changes:

```bash
npm run release:check
```

Local docs build:

```bash
npm run docs:build
```

After merge, repository Pages settings should use "GitHub Actions" as the Pages source.

## Star History

<p align="center">
  <a href="https://star-history.com/#tonycdr-prog/architect-mcp&Date">
    <img alt="Star History Chart for tonycdr-prog/architect-mcp" src="https://api.star-history.com/svg?repos=tonycdr-prog/architect-mcp&type=Date">
  </a>
</p>

## Security

Security reports should follow [SECURITY.md](SECURITY.md). Do not put vulnerabilities, secrets, exploit details, or private repository data in public issues.
