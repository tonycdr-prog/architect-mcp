import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerArchitectureTools } from "../tools/registerArchitectureTools.js";
import type { ToolSurface } from "../tools/toolRegistry.js";

export type ArchitectServerOptions = {
  enableLocalWorkspaceTool?: boolean;
  toolSurface?: ToolSurface;
};

export function createArchitectServer(options: ArchitectServerOptions = {}): McpServer {
  const server = new McpServer({
    name: "architect-mcp",
    version: "0.1.0"
  });

  registerArchitectureTools(server, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool ?? true,
    toolSurface: options.toolSurface ?? "core"
  });

  return server;
}

export function parseToolSurface(value: string | undefined): ToolSurface {
  if (!value) return "core";
  if (value === "core" || value === "advanced") return value;
  throw new Error(`Invalid ARCHITECT_MCP_TOOL_SURFACE value "${value}". Expected "core" or "advanced".`);
}
