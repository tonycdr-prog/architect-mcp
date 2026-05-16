import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";
import { reviewMcpConfigSecurity } from "./mcpSecurity.js";
import type {
  McpCatalogQuery,
  McpCatalogServer,
  McpInstallPlan,
  McpInstallTargetClient,
  McpRecommendation,
  McpRecommendationInput,
  McpServerCatalog
} from "./mcpCatalogTypes.js";

const mcpCatalogSchema = z.object({
  version: z.string().min(1),
  sources: z.array(z.object({
    label: z.string().min(1),
    url: z.string().url(),
    notes: z.string().min(1)
  }).strict()).min(1),
  servers: z.array(z.object({
    id: z.string().regex(/^[a-z0-9-]+$/),
    name: z.string().min(1),
    provider: z.string().min(1),
    category: z.enum(["database", "payments", "workspace", "deployment", "testing", "memory", "developer-tools"]),
    trustTier: z.enum(["vendor-official", "reference", "community-vetted", "unknown"]),
    transport: z.enum(["http", "stdio", "http-or-stdio", "research-only"]),
    aliases: z.array(z.string().min(1)).min(1),
    capabilitySignals: z.array(z.string().min(1)).min(1),
    requiresExplicitProvider: z.boolean(),
    requiresServerBoundary: z.boolean(),
    recommendedWhen: z.array(z.string().min(1)).min(1),
    decisionWarnings: z.array(z.string().min(1)),
    install: z.object({
      serverName: z.string().min(1),
      config: z.record(z.string(), z.unknown()),
      env: z.array(z.string().min(1)),
      postInstall: z.array(z.string().min(1))
    }).strict(),
    package: z.object({
      registry: z.string().min(1),
      name: z.string().min(1),
      version: z.string().min(1),
      license: z.string().min(1)
    }).strict()
  }).strict()).min(1)
}).strict();

let cachedCatalog: McpServerCatalog | undefined;

export function listMcpServerCatalog(query: McpCatalogQuery = {}) {
  const catalog = loadMcpServerCatalog();
  const normalizedQuery = query.query?.toLowerCase().trim();
  const normalizedProvider = query.provider?.toLowerCase().trim();
  const servers = catalog.servers.filter((server) => {
    if (query.category && server.category !== query.category) return false;
    if (query.trustTier && server.trustTier !== query.trustTier) return false;
    if (normalizedProvider && !server.provider.toLowerCase().includes(normalizedProvider)) return false;
    if (!normalizedQuery) return true;

    const haystack = [
      server.id,
      server.name,
      server.provider,
      server.category,
      server.trustTier,
      server.transport,
      ...server.aliases,
      ...server.capabilitySignals,
      ...server.recommendedWhen,
      ...server.decisionWarnings
    ].join(" ").toLowerCase();
    return haystack.includes(normalizedQuery);
  });

  return {
    version: catalog.version,
    sources: catalog.sources,
    servers
  };
}

export function recommendMcpServers(input: McpRecommendationInput = {}) {
  const catalog = loadMcpServerCatalog();
  const text = recommendationText(input);
  const confirmedProviders = new Set((input.confirmedProviders ?? []).map((provider) => provider.toLowerCase()));
  const questions: string[] = [];
  const recommendations: McpRecommendation[] = [];

  const hasGenericDatabaseNeed = /\b(database|db|postgres|postgresql|sql|auth|storage|realtime)\b/.test(text);
  const hasGenericPaymentNeed = /\b(payment|payments|billing|checkout|subscription|subscriptions|webhook|webhooks)\b/.test(text);
  const serverBoundaryConfirmed = input.serverSideBoundaryConfirmed === true || /\b(server-side|server side|backend|api route|webhook|webhooks|edge function|cloud function|secret boundary|server-owned|server owned)\b/.test(text);

  for (const server of catalog.servers) {
    if (server.transport === "research-only") continue;
    const explicit = isExplicitServerMention(server, text, confirmedProviders);

    if (server.id === "supabase") {
      if (explicit) recommendations.push(toRecommendation(server, "high", "Supabase was explicitly selected or already present in the project context."));
      continue;
    }

    if (server.id === "stripe") {
      if (explicit && serverBoundaryConfirmed) {
        recommendations.push(toRecommendation(server, "high", "Stripe was explicitly selected and a server-side payment boundary is confirmed."));
      }
      continue;
    }

    if (server.id === "playwright") {
      if (explicit || /\b(browser test|browser tests|e2e|end-to-end|visual smoke|playwright|ui test|ui tests)\b/.test(text)) {
        recommendations.push(toRecommendation(server, explicit ? "high" : "medium", "The project needs browser-level verification."));
      }
      continue;
    }

    if (server.id === "github") {
      if (explicit || /\b(github issue|github issues|pull request|pull requests|github actions)\b/.test(text)) {
        recommendations.push(toRecommendation(server, explicit ? "high" : "medium", "The project needs repository, issue, pull request, or CI operations."));
      }
      continue;
    }

    if (explicit) recommendations.push(toRecommendation(server, "high", `${server.provider} was explicitly selected or already present in the project context.`));
  }

  if (hasGenericDatabaseNeed && !isExplicitServerMention(catalogServer(catalog, "supabase"), text, confirmedProviders)) {
    questions.push("Which database or backend provider should this project use before adding an MCP server?");
  }

  if (hasGenericPaymentNeed && !isExplicitServerMention(catalogServer(catalog, "stripe"), text, confirmedProviders)) {
    questions.push("Which payment provider should this project use, and should the integration be server-side?");
  }

  if (isExplicitServerMention(catalogServer(catalog, "stripe"), text, confirmedProviders) && !serverBoundaryConfirmed) {
    questions.push("Confirm the server-side Stripe boundary before installing or recommending Stripe MCP.");
  }

  return {
    status: questions.length > 0 ? "needs-clarification" : recommendations.length > 0 ? "pass" : "no-match",
    questions,
    recommendations: dedupeRecommendations(recommendations),
    policy: [
      "Unknown MCP servers fail closed and require research before installation.",
      "Generic database needs trigger a provider question instead of defaulting to Supabase.",
      "Stripe recommendations require an explicit Stripe signal and a confirmed server-side payment boundary.",
      "Hosted mode may return recommendations, but only local tools can write local MCP client config."
    ]
  };
}

export function createMcpInstallPlan(input: {
  serverId: string;
  targetClient?: McpInstallTargetClient;
  hostedMode?: boolean;
}): McpInstallPlan {
  const server = findMcpServer(input.serverId);
  if (server.transport === "research-only" || Object.keys(server.install.config).length === 0) {
    throw new Error(`MCP server ${server.id} is research-only and does not have an install plan.`);
  }

  const targetClient = input.targetClient ?? "generic-json";
  const serverConfig = cloneRecord(server.install.config);
  const mcpConfig = {
    mcpServers: {
      [server.install.serverName]: serverConfig
    }
  };
  const hostedMode = input.hostedMode === true;
  const localOnly = server.transport === "stdio";

  return {
    id: `mcp-install-${server.id}`,
    serverId: server.id,
    serverName: server.install.serverName,
    targetClient,
    status: "dry-run",
    hostedMode,
    localOnly,
    requiresApproval: true,
    writeFiles: false,
    packagePin: packagePin(server),
    mcpConfig,
    clientConfig: clientConfigFor(targetClient, server.install.serverName, serverConfig),
    env: server.install.env,
    postInstall: server.install.postInstall,
    warnings: [
      ...server.decisionWarnings,
      ...(hostedMode && localOnly ? ["Hosted mode can recommend this server, but cannot write or run a local stdio MCP configuration."] : [])
    ]
  };
}

export function reviewMcpInstallPlan(input: { plan: McpInstallPlan; writeFiles?: boolean; explicitApproval?: boolean }) {
  const findings: Array<{ code: string; severity: "error" | "warning"; message: string; recommendation: string }> = [];
  let server: McpCatalogServer | undefined;
  try {
    server = findMcpServer(input.plan.serverId);
  } catch {
    findings.push({
      code: "MCPINSTALL001_UNKNOWN_SERVER",
      severity: "error",
      message: `Unknown MCP server id: ${input.plan.serverId}.`,
      recommendation: "Research and add the server to the local catalog before creating an install plan."
    });
  }

  if (server && input.plan.serverName !== server.install.serverName) {
    findings.push({
      code: "MCPINSTALL002_SERVER_NAME_DRIFT",
      severity: "error",
      message: `Install plan server name ${input.plan.serverName} does not match catalog entry ${server.install.serverName}.`,
      recommendation: "Regenerate the install plan from the catalog entry."
    });
  }

  const security = reviewMcpConfigSecurity({
    config: input.plan.mcpConfig,
    approvedServers: [input.plan.serverName]
  });
  for (const finding of security.findings) {
    findings.push({
      code: finding.code,
      severity: finding.severity === "critical" || finding.severity === "high" ? "error" : "warning",
      message: finding.message,
      recommendation: finding.recommendation
    });
  }

  if (containsLatest(input.plan.clientConfig) || containsLatest(input.plan.mcpConfig)) {
    findings.push({
      code: "MCPINSTALL003_UNPINNED_PACKAGE",
      severity: "error",
      message: "Install plan contains @latest or an unpinned package reference.",
      recommendation: "Pin package-backed MCP servers to an exact reviewed version."
    });
  }

  if (input.writeFiles && !input.explicitApproval) {
    findings.push({
      code: "MCPINSTALL004_MISSING_WRITE_APPROVAL",
      severity: "error",
      message: "Install plan writes files without explicit approval.",
      recommendation: "Keep the plan in dry-run mode or pass explicitApproval only after user confirmation."
    });
  }

  if (input.writeFiles && input.plan.hostedMode) {
    findings.push({
      code: "MCPINSTALL005_HOSTED_WRITE",
      severity: "error",
      message: "Hosted mode cannot write local MCP client configuration.",
      recommendation: "Return the install plan for user review and apply it locally from a local tool surface."
    });
  }

  return {
    status: findings.some((finding) => finding.severity === "error") ? "fail" : findings.length > 0 ? "warn" : "pass",
    findings,
    security
  };
}

export function applyMcpInstallPlan(input: {
  plan: McpInstallPlan;
  targetPath?: string;
  writeFiles?: boolean;
  explicitApproval?: boolean;
}) {
  const targetPath = resolveLocalTargetPath(input.targetPath ?? ".mcp.json");
  const review = reviewMcpInstallPlan({
    plan: input.plan,
    writeFiles: input.writeFiles,
    explicitApproval: input.explicitApproval
  });
  if (review.status === "fail") {
    return {
      status: "blocked",
      targetPath,
      review
    };
  }

  if (!input.writeFiles) {
    return {
      status: "dry-run",
      targetPath,
      review,
      files: [
        {
          path: targetPath,
          operation: existsSync(targetPath) ? "merge" : "create",
          content: `${JSON.stringify(mergeClientConfig(readExistingJson(targetPath), input.plan.clientConfig), null, 2)}\n`
        }
      ]
    };
  }

  if (!input.explicitApproval) {
    return {
      status: "blocked",
      targetPath,
      review
    };
  }

  mkdirSync(dirname(targetPath), { recursive: true });
  const merged = mergeClientConfig(readExistingJson(targetPath), input.plan.clientConfig);
  writeFileSync(targetPath, `${JSON.stringify(merged, null, 2)}\n`, "utf8");
  return {
    status: "written",
    targetPath,
    review
  };
}

export function validateMcpServerCatalog() {
  const errors: string[] = [];
  try {
    const catalog = loadMcpServerCatalog();
    const ids = new Set<string>();
    for (const server of catalog.servers) {
      if (ids.has(server.id)) errors.push(`Duplicate MCP server id: ${server.id}`);
      ids.add(server.id);
      if (server.package.registry === "npm" && server.package.version === "latest") {
        errors.push(`${server.id}: npm package must be pinned to an exact version.`);
      }
      if (server.install.config && containsLatest(server.install.config)) {
        errors.push(`${server.id}: install config must not use @latest.`);
      }
      if (server.category === "database" && !server.requiresExplicitProvider) {
        errors.push(`${server.id}: database catalog entries must require explicit provider selection.`);
      }
      if (server.category === "payments" && !server.requiresServerBoundary) {
        errors.push(`${server.id}: payment catalog entries must require server-boundary confirmation.`);
      }
    }
  } catch (error) {
    errors.push(error instanceof Error ? error.message : String(error));
  }

  return {
    valid: errors.length === 0,
    errors,
    serverCount: errors.length === 0 ? loadMcpServerCatalog().servers.length : 0
  };
}

function loadMcpServerCatalog(): McpServerCatalog {
  cachedCatalog ??= mcpCatalogSchema.parse(JSON.parse(readFileSync(resolveMcpCatalogPath(), "utf8"))) as McpServerCatalog;
  return cachedCatalog;
}

function resolveMcpCatalogPath(): string {
  const candidates = [
    resolve(process.cwd(), "mcp-catalog/servers.json"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../mcp-catalog/servers.json"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../../mcp-catalog/servers.json")
  ];

  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`Could not resolve mcp-catalog/servers.json. Tried: ${candidates.join(", ")}`);
}

function recommendationText(input: McpRecommendationInput): string {
  return [
    input.request,
    input.projectBrief?.idea,
    input.projectBrief?.users,
    ...(input.projectBrief?.coreFlows ?? []),
    ...(input.projectBrief?.constraints ?? []),
    ...Object.values(input.projectBrief?.stack ?? {}).filter(Boolean),
    ...(input.selectedStackPacks ?? []),
    ...(input.confirmedProviders ?? [])
  ].join(" ").toLowerCase();
}

function isExplicitServerMention(server: McpCatalogServer, text: string, confirmedProviders: Set<string>): boolean {
  const explicitNames = [server.id, server.name, server.provider].map((value) => value.toLowerCase());
  return explicitNames.some((name) => text.includes(name)) || confirmedProviders.has(server.id) || confirmedProviders.has(server.provider.toLowerCase());
}

function toRecommendation(server: McpCatalogServer, confidence: McpRecommendation["confidence"], rationale: string): McpRecommendation {
  return {
    serverId: server.id,
    name: server.name,
    provider: server.provider,
    category: server.category,
    confidence,
    rationale,
    installMode: server.transport,
    requiredEnv: server.install.env,
    warnings: server.decisionWarnings
  };
}

function dedupeRecommendations(recommendations: McpRecommendation[]): McpRecommendation[] {
  const seen = new Set<string>();
  return recommendations.filter((recommendation) => {
    if (seen.has(recommendation.serverId)) return false;
    seen.add(recommendation.serverId);
    return true;
  });
}

function catalogServer(catalog: McpServerCatalog, id: string): McpCatalogServer {
  const server = catalog.servers.find((candidate) => candidate.id === id);
  if (!server) throw new Error(`Missing required MCP catalog entry: ${id}`);
  return server;
}

function findMcpServer(id: string): McpCatalogServer {
  const server = loadMcpServerCatalog().servers.find((candidate) => candidate.id === id);
  if (!server) throw new Error(`Unknown MCP server id: ${id}`);
  return server;
}

function cloneRecord(value: Record<string, unknown>): Record<string, unknown> {
  return JSON.parse(JSON.stringify(value)) as Record<string, unknown>;
}

function packagePin(server: McpCatalogServer): string {
  return `${server.package.registry}:${server.package.name}@${server.package.version}`;
}

function clientConfigFor(targetClient: McpInstallTargetClient, serverName: string, serverConfig: Record<string, unknown>): Record<string, unknown> {
  if (targetClient === "vscode") {
    return {
      servers: {
        [serverName]: serverConfig
      }
    };
  }
  return {
    mcpServers: {
      [serverName]: serverConfig
    }
  };
}

function containsLatest(value: unknown): boolean {
  if (typeof value === "string") return value.includes("@latest");
  if (Array.isArray(value)) return value.some(containsLatest);
  if (value && typeof value === "object") return Object.values(value).some(containsLatest);
  return false;
}

function resolveLocalTargetPath(targetPath: string): string {
  const workspaceRoot = realpathSync(process.cwd());
  const resolved = resolve(process.cwd(), targetPath);
  const parent = dirname(resolved);
  const realParent = realpathSync(nearestExistingParent(parent));
  if (realParent !== workspaceRoot && !realParent.startsWith(`${workspaceRoot}${sep}`)) {
    throw new Error(`Refusing to write MCP config outside the current workspace: ${targetPath}`);
  }
  return resolved;
}

function nearestExistingParent(path: string): string {
  let current = path;
  while (!existsSync(current)) {
    const next = dirname(current);
    if (next === current) return current;
    current = next;
  }
  return current;
}

function readExistingJson(path: string): Record<string, unknown> {
  if (!existsSync(path)) return {};
  const content = readFileSync(path, "utf8").trim();
  if (!content) return {};
  const parsed = JSON.parse(content) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`Existing MCP config is not a JSON object: ${path}`);
  }
  return parsed as Record<string, unknown>;
}

function mergeClientConfig(existing: Record<string, unknown>, next: Record<string, unknown>): Record<string, unknown> {
  const merged = cloneRecord(existing);
  for (const [sectionName, sectionValue] of Object.entries(next)) {
    if (!sectionValue || typeof sectionValue !== "object" || Array.isArray(sectionValue)) {
      merged[sectionName] = sectionValue;
      continue;
    }
    const existingSection = merged[sectionName];
    if (existingSection && typeof existingSection === "object" && !Array.isArray(existingSection)) {
      merged[sectionName] = {
        ...(existingSection as Record<string, unknown>),
        ...(sectionValue as Record<string, unknown>)
      };
    } else {
      merged[sectionName] = sectionValue;
    }
  }
  return merged;
}
