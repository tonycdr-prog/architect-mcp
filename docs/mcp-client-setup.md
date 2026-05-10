# MCP Client Setup

architect-mcp supports stdio MCP clients and a stateless hosted HTTP entrypoint.

## Release Tarball

Use the release tarball when configuring a client that should not depend on a source checkout:

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
        "https://github.com/tonycdr-prog/architect-mcp/releases/download/v0.1.0/tonycdr-prog-architect-mcp-0.1.0.tgz",
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
