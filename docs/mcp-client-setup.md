# MCP Client Setup

architect-mcp supports stdio MCP clients and a stateless hosted HTTP entrypoint.

## npm Package

Use the published package when configuring a client that should not depend on a source checkout:

```json
{
  "mcpServers": {
    "architect-mcp": {
      "command": "npx",
      "args": [
        "-y",
        "--package",
        "@tonycdr-prog/architect-mcp@latest",
        "architect-mcp"
      ]
    }
  }
}
```

## Source Checkout

Build first, then point the client at your absolute path to `dist/index.js`:

```json
{
  "mcpServers": {
    "architect-mcp": {
      "command": "node",
      "args": ["/absolute/path/to/architect-mcp/dist/index.js"]
    }
  }
}
```

## Advanced Surface

Set `ARCHITECT_MCP_TOOL_SURFACE=advanced` in the MCP server environment:

```json
{
  "mcpServers": {
    "architect-mcp": {
      "command": "npx",
      "args": [
        "-y",
        "--package",
        "@tonycdr-prog/architect-mcp@latest",
        "architect-mcp"
      ],
      "env": {
        "ARCHITECT_MCP_TOOL_SURFACE": "advanced"
      }
    }
  }
}
```

## Hosted HTTP

Hosted HTTP mode is stateless. Each `/mcp` request creates a fresh server instance and excludes local-only tools.

```bash
npm install
npm run build
PORT=3000 HOST=0.0.0.0 npm run start:http
```

Endpoints:

- `POST /mcp`: Streamable HTTP MCP endpoint.
- `GET /health`: platform health check.

Use `review_repo_structure` with explicit file summaries in hosted mode. Do not send arbitrary server-local paths.

## Additional MCP Servers

The advanced surface includes guarded MCP integration tools:

- `list_mcp_server_catalog`
- `recommend_mcp_servers`
- `create_mcp_install_plan`
- `review_mcp_install_plan`
- `apply_mcp_install_plan`

Use them after the core work gate has clarified providers and boundaries. Database needs ask for a provider instead of defaulting to Supabase. Stripe needs an explicit Stripe signal plus a server-side payment boundary. `apply_mcp_install_plan` is local-only, dry-run by default, and writes only project-local JSON config after explicit approval.

See [MCP Integrations](./mcp-integrations.md).
