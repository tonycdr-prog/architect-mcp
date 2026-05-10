#!/usr/bin/env node
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { createArchitectServer, parseToolSurface } from "./server/createArchitectServer.js";

const server = createArchitectServer({
  enableLocalWorkspaceTool: true,
  toolSurface: parseToolSurface(process.env.ARCHITECT_MCP_TOOL_SURFACE)
});

const transport = new StdioServerTransport();
await server.connect(transport);
