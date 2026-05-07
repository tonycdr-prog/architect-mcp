import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join, resolve } from "node:path";
import { reviewMcpConfigSecurity } from "./mcpSecurity.js";

export type McpConfigScanInput = {
  rootPath?: string;
  includeUserConfig?: boolean;
  approvedServers?: string[];
};

export function scanMcpConfigFiles(input: McpConfigScanInput = {}) {
  const paths = candidateConfigPaths(input.rootPath, input.includeUserConfig);
  const results = paths.map((path) => scanOne(path, input.approvedServers));
  return {
    scanned: results.filter((result) => result.status !== "missing").length,
    missing: results.filter((result) => result.status === "missing").length,
    results
  };
}

function candidateConfigPaths(rootPath = process.cwd(), includeUserConfig = false): string[] {
  const root = resolve(rootPath);
  const paths = [
    join(root, ".mcp.json"),
    join(root, ".cursor", "mcp.json"),
    join(root, ".cursor", "mcp.jsonc"),
    join(root, ".codex", "mcp.json")
  ];

  if (includeUserConfig) {
    const home = homedir();
    paths.push(
      join(home, "Library", "Application Support", "Claude", "claude_desktop_config.json"),
      join(home, ".cursor", "mcp.json"),
      join(home, ".codex", "mcp.json")
    );
  }

  return [...new Set(paths)];
}

function scanOne(path: string, approvedServers?: string[]) {
  if (!existsSync(path)) {
    return {
      path,
      status: "missing" as const
    };
  }

  try {
    const config = parseJsonLike(readFileSync(path, "utf8"));
    return {
      path,
      status: "reviewed" as const,
      review: reviewMcpConfigSecurity({ config, approvedServers })
    };
  } catch (error) {
    return {
      path,
      status: "error" as const,
      error: error instanceof Error ? error.message : String(error)
    };
  }
}

function parseJsonLike(content: string): unknown {
  return JSON.parse(stripJsonComments(content));
}

function stripJsonComments(content: string): string {
  let output = "";
  let inString = false;
  let quote = "";
  let escaped = false;
  let inLineComment = false;
  let inBlockComment = false;

  for (let index = 0; index < content.length; index += 1) {
    const char = content[index] ?? "";
    const next = content[index + 1] ?? "";

    if (inLineComment) {
      if (char === "\n" || char === "\r") {
        inLineComment = false;
        output += char;
      }
      continue;
    }

    if (inBlockComment) {
      if (char === "*" && next === "/") {
        inBlockComment = false;
        index += 1;
        continue;
      }
      output += char === "\n" || char === "\r" ? char : " ";
      continue;
    }

    if (inString) {
      output += char;
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        inString = false;
        quote = "";
      }
      continue;
    }

    if (char === "\"" || char === "'") {
      inString = true;
      quote = char;
      output += char;
      continue;
    }

    if (char === "/" && next === "/") {
      inLineComment = true;
      index += 1;
      continue;
    }

    if (char === "/" && next === "*") {
      inBlockComment = true;
      output += "  ";
      index += 1;
      continue;
    }

    output += char;
  }

  return output;
}
