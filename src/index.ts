#!/usr/bin/env node
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { createArchitectServer } from "./server/createArchitectServer.js";

const server = createArchitectServer({
  enableLocalWorkspaceTool: true
});

const transport = new StdioServerTransport();
await server.connect(transport);
