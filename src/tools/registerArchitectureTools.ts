import type { McpServer, RegisteredTool } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerArtifactTools } from "./artifactTools.js";
import { registerContractTools } from "./contractTools.js";
import { registerFoundryTools } from "./foundryTools.js";
import { registerHarnessTools } from "./harnessTools.js";
import { registerIntakeTools } from "./intakeTools.js";
import { registerLayoutTools } from "./layoutTools.js";
import { registerMemoryTools } from "./memoryTools.js";
import { registerMcpCatalogTools } from "./mcpCatalogTools.js";
import { registerPackTools } from "./packTools.js";
import { registerProfileTools } from "./profileTools.js";
import { registerReviewTools } from "./reviewTools.js";
import { registerSkillCatalogTools } from "./skillCatalogTools.js";
import { registerValidationTools } from "./validationTools.js";
import { registerV3Tools } from "./v3Tools.js";
import { registerV5V9Tools } from "./v5V9Tools.js";
import { registerV10Tools } from "./v10Tools.js";
import { registerRepoQualityEvalTools } from "./repoQualityEvalTools.js";
import { isArchitectureToolEnabled, type ToolSurface } from "./toolRegistry.js";

export type RegisterArchitectureToolsOptions = {
  enableLocalWorkspaceTool?: boolean;
  toolSurface?: ToolSurface;
};

export function registerArchitectureTools(server: McpServer, options: RegisterArchitectureToolsOptions = {}): void {
  const filteredServer = createToolSurfaceServer(server, {
    includeLocal: options.enableLocalWorkspaceTool ?? true,
    surface: options.toolSurface ?? "core"
  });

  registerPackTools(filteredServer);
  registerProfileTools(filteredServer);
  registerLayoutTools(filteredServer);
  registerIntakeTools(filteredServer);
  registerHarnessTools(filteredServer);
  registerMemoryTools(filteredServer);
  registerMcpCatalogTools(filteredServer);
  registerSkillCatalogTools(filteredServer);
  registerContractTools(filteredServer);
  registerArtifactTools(filteredServer);
  registerV3Tools(filteredServer, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool,
    toolSurface: options.toolSurface ?? "core"
  });
  registerV5V9Tools(filteredServer);
  registerV10Tools(filteredServer);
  registerRepoQualityEvalTools(filteredServer);
  registerFoundryTools(filteredServer, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool
  });
  registerReviewTools(filteredServer, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool
  });
  registerValidationTools(filteredServer);
}

function createToolSurfaceServer(server: McpServer, options: { includeLocal: boolean; surface: ToolSurface }): McpServer {
  const originalRegisterTool = server.registerTool.bind(server);
  const filteredServer = Object.create(server) as McpServer;
  filteredServer.registerTool = ((name, config, cb) => {
    if (!isArchitectureToolEnabled(name, options)) {
      return skippedRegisteredTool(config);
    }
    return originalRegisterTool(name, config, cb);
  }) as McpServer["registerTool"];
  return filteredServer;
}

function skippedRegisteredTool(config: { title?: string; description?: string; inputSchema?: unknown; outputSchema?: unknown; annotations?: unknown; _meta?: Record<string, unknown> }): RegisteredTool {
  return {
    title: config.title,
    description: config.description,
    inputSchema: config.inputSchema as RegisteredTool["inputSchema"],
    outputSchema: config.outputSchema as RegisteredTool["outputSchema"],
    annotations: config.annotations as RegisteredTool["annotations"],
    _meta: config._meta,
    handler: async () => ({
      content: []
    }),
    enabled: false,
    enable() {
      this.enabled = true;
    },
    disable() {
      this.enabled = false;
    },
    update(updates) {
      Object.assign(this, updates);
    },
    remove() {
      this.enabled = false;
    }
  };
}
