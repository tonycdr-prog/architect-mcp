import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerArtifactTools } from "./artifactTools.js";
import { registerContractTools } from "./contractTools.js";
import { registerHarnessTools } from "./harnessTools.js";
import { registerIntakeTools } from "./intakeTools.js";
import { registerLayoutTools } from "./layoutTools.js";
import { registerMemoryTools } from "./memoryTools.js";
import { registerPackTools } from "./packTools.js";
import { registerProfileTools } from "./profileTools.js";
import { registerReviewTools } from "./reviewTools.js";
import { registerSkillCatalogTools } from "./skillCatalogTools.js";
import { registerValidationTools } from "./validationTools.js";
import { registerV3Tools } from "./v3Tools.js";
import { registerV5V9Tools } from "./v5V9Tools.js";
import { registerV10Tools } from "./v10Tools.js";
import { registerRepoQualityEvalTools } from "./repoQualityEvalTools.js";

export type RegisterArchitectureToolsOptions = {
  enableLocalWorkspaceTool?: boolean;
};

export function registerArchitectureTools(server: McpServer, options: RegisterArchitectureToolsOptions = {}): void {
  registerPackTools(server);
  registerProfileTools(server);
  registerLayoutTools(server);
  registerIntakeTools(server);
  registerHarnessTools(server);
  registerMemoryTools(server);
  registerSkillCatalogTools(server);
  registerContractTools(server);
  registerArtifactTools(server);
  registerV3Tools(server, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool
  });
  registerV5V9Tools(server);
  registerV10Tools(server);
  registerRepoQualityEvalTools(server);
  registerReviewTools(server, {
    enableLocalWorkspaceTool: options.enableLocalWorkspaceTool
  });
  registerValidationTools(server);
}
