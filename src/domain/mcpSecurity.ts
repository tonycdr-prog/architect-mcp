export type McpSecuritySeverity = "critical" | "high" | "medium" | "low";

export type McpSecurityFinding = {
  code:
    | "MCPSEC001_HARDCODED_SECRET"
    | "MCPSEC002_SHELL_INJECTION"
    | "MCPSEC003_UNPINNED_DEPENDENCY"
    | "MCPSEC004_UNAPPROVED_SERVER"
    | "MCPSEC005_INTERACTIVE_NPX"
    | "MCPSEC006_RISKY_COMMAND";
  severity: McpSecuritySeverity;
  server?: string;
  message: string;
  evidence: string;
  recommendation: string;
};

export type McpSecurityReviewInput = {
  config: unknown;
  approvedServers?: string[];
  allowShell?: boolean;
};

const secretPatterns: Array<[RegExp, string]> = [
  [/(api[_-]?key|token|secret|password|credential)["']?\s*[:=]\s*["'](?!\$\{)[^"']{8,}/i, "Secret-like key/value pair"],
  [/Bearer\s+[A-Za-z0-9\-._~+/]+=*/i, "Bearer token"],
  [/(ghp_|gho_|ghu_|ghs_|ghr_)[A-Za-z0-9]{30,}/, "GitHub token"],
  [/sk-[A-Za-z0-9]{20,}/, "OpenAI-style API key"],
  [/AKIA[0-9A-Z]{16}/, "AWS access key"],
  [/-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----/i, "Private key"],
  [/[a-z]+:\/\/[^:\s]+:[^@\s]{6,}@/i, "Credential in connection URL"]
];

const shellPatterns: Array<[RegExp, string]> = [
  [/\$\(/, "Command substitution $(...)"],
  [/`[^`]+`/, "Backtick command substitution"],
  [/;\s*\w/, "Command chaining with semicolon"],
  [/\|\s*\w/, "Pipe to another command"],
  [/&&\s*\w/, "Command chaining with &&"],
  [/\|\|\s*\w/, "Command chaining with ||"],
  [/\beval\s/i, "eval usage"],
  [/\b(?:bash|sh)\s+-c\b/i, "Shell command execution"],
  [/>\\?\s*\/dev\/tcp\//i, "TCP redirect pattern"],
  [/curl\s+.*\|\s*(ba)?sh/i, "curl pipe to shell"]
];

export function reviewMcpConfigSecurity(input: McpSecurityReviewInput) {
  const config = normalizeConfig(input.config);
  const servers = config.mcpServers;
  const findings: McpSecurityFinding[] = [];

  const raw = JSON.stringify(config);
  for (const [pattern, label] of secretPatterns) {
    if (pattern.test(raw)) {
      findings.push({
        code: "MCPSEC001_HARDCODED_SECRET",
        severity: "critical",
        message: `${label} found in MCP configuration.`,
        evidence: pattern.source,
        recommendation: "Move credentials to environment variables and reference them by name."
      });
    }
  }

  const approved = new Set(input.approvedServers?.map((server) => server.toLowerCase()) ?? []);
  for (const [name, server] of Object.entries(servers)) {
    if (!isRecord(server)) {
      findings.push({
        code: "MCPSEC006_RISKY_COMMAND",
        severity: "high",
        server: name,
        message: "MCP server entry is not an object.",
        evidence: typeof server,
        recommendation: "Use an object with command, args, and env fields."
      });
      continue;
    }

    if (approved.size > 0 && !approved.has(name.toLowerCase())) {
      findings.push({
        code: "MCPSEC004_UNAPPROVED_SERVER",
        severity: "medium",
        server: name,
        message: `MCP server ${name} is not on the approved list.`,
        evidence: name,
        recommendation: "Approve the server explicitly or remove it from the configuration."
      });
    }

    const command = stringValue(server.command);
    const args = Array.isArray(server.args) ? server.args.map(String) : [];
    const argsText = JSON.stringify(args);

    if (!input.allowShell) {
      for (const [pattern, label] of shellPatterns) {
        if (pattern.test(argsText) || pattern.test(command)) {
          findings.push({
            code: "MCPSEC002_SHELL_INJECTION",
            severity: "high",
            server: name,
            message: `Dangerous shell pattern in MCP server command or args: ${label}.`,
            evidence: label,
            recommendation: "Use direct command execution and pass fixed arguments without shell interpolation."
          });
        }
      }
    }

    if (/\b(?:bash|sh|zsh|fish)\b/.test(command)) {
      findings.push({
        code: "MCPSEC006_RISKY_COMMAND",
        severity: "high",
        server: name,
        message: "MCP server starts through a shell.",
        evidence: command,
        recommendation: "Prefer direct executable commands such as node, npx, uvx, docker, or a pinned binary."
      });
    }

    if (command === "npx" && !args.includes("-y")) {
      findings.push({
        code: "MCPSEC005_INTERACTIVE_NPX",
        severity: "low",
        server: name,
        message: "npx without -y may prompt interactively.",
        evidence: args.join(" "),
        recommendation: "Add -y and pin the package version."
      });
    }

    for (const arg of args) {
      if (arg.includes("@latest") || looksLikeUnpinnedPackage(command, arg)) {
        findings.push({
          code: "MCPSEC003_UNPINNED_DEPENDENCY",
          severity: "medium",
          server: name,
          message: `MCP server dependency is not pinned: ${arg}.`,
          evidence: arg,
          recommendation: "Pin MCP server packages to an exact version and review upgrades intentionally."
        });
      }
    }
  }

  return {
    status: findings.some((finding) => finding.severity === "critical" || finding.severity === "high")
      ? "fail"
      : findings.length > 0 ? "warn" : "pass",
    summary: {
      servers: Object.keys(servers).length,
      critical: count(findings, "critical"),
      high: count(findings, "high"),
      medium: count(findings, "medium"),
      low: count(findings, "low")
    },
    findings
  };
}

function normalizeConfig(config: unknown): { mcpServers: Record<string, unknown> } {
  if (!isRecord(config)) throw new Error("MCP config must be a JSON object.");
  const servers = config.mcpServers;
  if (!isRecord(servers)) throw new Error("MCP config must include an mcpServers object.");
  return { mcpServers: servers };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function stringValue(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function looksLikeUnpinnedPackage(command: string, arg: string): boolean {
  if (command !== "npx" && command !== "uvx") return false;
  if (arg.startsWith("-") || arg.startsWith(".") || arg.startsWith("/")) return false;
  if (!/^[a-z0-9@/_.~^-]+$/i.test(arg)) return false;
  const version = packageVersion(arg);
  return !version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version);
}

function packageVersion(arg: string): string | undefined {
  const match = arg.startsWith("@")
    ? /^@[^/]+\/[^@]+@(.+)$/.exec(arg)
    : /^[^@]+@(.+)$/.exec(arg);
  return match?.[1];
}

function count(findings: McpSecurityFinding[], severity: McpSecuritySeverity): number {
  return findings.filter((finding) => finding.severity === severity).length;
}
