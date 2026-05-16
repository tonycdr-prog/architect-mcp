export type McpServerCategory =
  | "database"
  | "payments"
  | "workspace"
  | "deployment"
  | "testing"
  | "memory"
  | "developer-tools";

export type McpServerTrustTier = "vendor-official" | "reference" | "community-vetted" | "unknown";
export type McpServerTransport = "http" | "stdio" | "http-or-stdio" | "research-only";

export type McpCatalogSource = {
  label: string;
  url: string;
  notes: string;
};

export type McpCatalogServer = {
  id: string;
  name: string;
  provider: string;
  category: McpServerCategory;
  trustTier: McpServerTrustTier;
  transport: McpServerTransport;
  aliases: string[];
  capabilitySignals: string[];
  requiresExplicitProvider: boolean;
  requiresServerBoundary: boolean;
  recommendedWhen: string[];
  decisionWarnings: string[];
  install: {
    serverName: string;
    config: Record<string, unknown>;
    env: string[];
    postInstall: string[];
  };
  package: {
    registry: string;
    name: string;
    version: string;
    license: string;
  };
};

export type McpServerCatalog = {
  version: string;
  sources: McpCatalogSource[];
  servers: McpCatalogServer[];
};

export type McpCatalogQuery = {
  category?: McpServerCategory;
  provider?: string;
  trustTier?: McpServerTrustTier;
  query?: string;
};

export type McpRecommendationInput = {
  request?: string;
  projectBrief?: {
    idea?: string;
    users?: string;
    coreFlows?: string[];
    stack?: Record<string, string | undefined>;
    constraints?: string[];
  };
  selectedStackPacks?: string[];
  confirmedProviders?: string[];
  serverSideBoundaryConfirmed?: boolean;
};

export type McpRecommendation = {
  serverId: string;
  name: string;
  provider: string;
  category: McpServerCategory;
  confidence: "high" | "medium" | "low";
  rationale: string;
  installMode: McpServerTransport;
  requiredEnv: string[];
  warnings: string[];
};

export type McpInstallTargetClient = "codex" | "claude-code" | "cursor" | "vscode" | "generic-json";

export type McpInstallPlan = {
  id: string;
  serverId: string;
  serverName: string;
  targetClient: McpInstallTargetClient;
  status: "dry-run";
  hostedMode: boolean;
  localOnly: boolean;
  requiresApproval: true;
  writeFiles: false;
  packagePin: string;
  mcpConfig: { mcpServers: Record<string, Record<string, unknown>> };
  clientConfig: Record<string, unknown>;
  env: string[];
  postInstall: string[];
  warnings: string[];
};
