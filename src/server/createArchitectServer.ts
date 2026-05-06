import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerArchitectureTools } from "../tools/registerArchitectureTools.js";

export type ArchitectServerOptions = {
  enableLocalWorkspaceTool?: boolean;
};

export function createArchitectServer(options: ArchitectServerOptions = {}): McpServer {
  const server = new McpServer({
    name: "architect-mcp",
    version: "0.1.0"
  });

  registerArchitectureTools(server, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool ?? true
  });

  return server;
}
