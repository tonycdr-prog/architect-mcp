# MCP Integrations

architect-mcp can recommend additional MCP servers, but it does not auto-install arbitrary tools from a vague brief. The integration flow is catalog-first and fail-closed.

## Flow

1. Use the core work gate to clarify the app idea and stack.
2. Call `recommend_mcp_servers` with the project brief and any confirmed providers.
3. If the response asks provider or boundary questions, answer those before installing anything.
4. Call `create_mcp_install_plan` for a selected catalog server.
5. Call `review_mcp_install_plan`.
6. Only local clients may call `apply_mcp_install_plan`; it is dry-run by default and writes only when `writeFiles: true` and explicit approval are supplied.

## TUI Flow

The Rust TUI command palette wraps the same tools:

```text
integrations recommend [extra context]
integrations plan <server-id> [target-client]
integrations review
integrations apply [target-path]
integrations approve <reason>
integrations write [target-path]
```

The TUI stores recommendation, install-plan, and review evidence in the session. A server cannot be planned unless it appears in the latest recommendation response. `integrations apply` is dry-run only. `integrations write` requires a non-failing review plus `integrations approve <reason>` and consumes that approval after a successful write.

## Policy

- Unknown MCP servers are not safe by default. Add them to the local catalog with source, package, transport, credential, and security notes first.
- Generic database needs ask for a provider. They do not default to Supabase.
- Explicit Supabase needs can recommend Supabase MCP.
- Stripe/payment needs require an explicit Stripe signal and a confirmed server-side payment boundary.
- Hosted mode may recommend MCP servers, but it cannot write local MCP client files.
- Install plans must not contain secrets, production credentials, or `@latest` packages.

## Catalog Sources

The first catalog is intentionally small and source-backed:

- Official MCP Registry: <https://modelcontextprotocol.io/registry/about>
- Official MCP reference servers: <https://github.com/modelcontextprotocol/servers>
- Supabase MCP: <https://supabase.com/docs/guides/getting-started/mcp>
- Stripe MCP: <https://docs.stripe.com/mcp>
- Notion MCP: <https://developers.notion.com/guides/mcp/get-started-with-mcp>
- Railway MCP: <https://docs.railway.com/cli/mcp>
- CLI Printing Press: <https://github.com/mvanhorn/cli-printing-press>
- Printing Press Library: <https://github.com/mvanhorn/printing-press-library>

The Printing Press repos are useful ecosystem research because they pair generated CLIs and MCP servers from a shared spec. They are not auto-installed by default; individual generated tools still need package, credential, provenance, and license review.

## Local Apply Boundary

`apply_mcp_install_plan` is local-only. It can merge a reviewed plan into a project-local JSON MCP config such as `.mcp.json`, but it refuses paths outside the current workspace. Use dry-run output for user-level config files or clients that store MCP configuration outside the repo.
