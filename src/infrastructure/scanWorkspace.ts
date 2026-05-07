import { readdir, readFile, stat } from "node:fs/promises";
import { join, relative } from "node:path";
import ts from "typescript";
import { normalizePath, pathPatternToRegExp } from "../domain/pathRules.js";
import type { FileSummary } from "../domain/types.js";

const DEFAULT_IGNORES = new Set([
  ".git",
  "dist",
  "node_modules",
  ".next",
  "coverage",
  ".turbo",
  ".cache"
]);

export type WorkspaceScanResult = {
  files: FileSummary[];
  truncated: boolean;
  maxFiles: number;
  ignoredPatterns: string[];
};

export async function scanWorkspace(rootPath: string, maxFiles: number, options: { ignorePatterns?: string[] } = {}): Promise<FileSummary[]> {
  return (await scanWorkspaceWithMetadata(rootPath, maxFiles, options)).files;
}

export async function scanWorkspaceWithMetadata(rootPath: string, maxFiles: number, options: { ignorePatterns?: string[] } = {}): Promise<WorkspaceScanResult> {
  const summaries: FileSummary[] = [];
  const state = { truncated: false };
  const ignoredPatterns = options.ignorePatterns ?? [];
  const compiledIgnores = ignoredPatterns.map(compileIgnorePattern);
  await walk(rootPath, rootPath, summaries, maxFiles, compiledIgnores, state);
  return {
    files: summaries,
    truncated: state.truncated,
    maxFiles,
    ignoredPatterns
  };
}

type CompiledIgnorePattern = {
  normalized: string;
  regex: RegExp;
  directoryRegex: RegExp;
  prefix?: string;
};

async function walk(rootPath: string, currentPath: string, summaries: FileSummary[], maxFiles: number, ignorePatterns: CompiledIgnorePattern[], state: { truncated: boolean }): Promise<void> {
  if (summaries.length >= maxFiles) {
    state.truncated = true;
    return;
  }

  const entries = await readdir(currentPath, { withFileTypes: true });
  for (const entry of entries) {
    if (summaries.length >= maxFiles) {
      state.truncated = true;
      return;
    }
    if (DEFAULT_IGNORES.has(entry.name)) continue;

    const absolutePath = join(currentPath, entry.name);
    const relativePath = normalizePath(relative(rootPath, absolutePath));
    if (isIgnored(relativePath, entry.isDirectory(), ignorePatterns)) continue;
    if (entry.isDirectory()) {
      await walk(rootPath, absolutePath, summaries, maxFiles, ignorePatterns, state);
      continue;
    }

    if (!entry.isFile()) continue;
    const metadata = await stat(absolutePath);
    summaries.push({
      path: relativePath,
      bytes: metadata.size,
      ...(await summarizeContent(absolutePath, metadata.size))
    });
  }
}

function compileIgnorePattern(pattern: string): CompiledIgnorePattern {
  const normalized = normalizePath(pattern);
  return {
    normalized,
    regex: pathPatternToRegExp(normalized),
    directoryRegex: pathPatternToRegExp(normalized.endsWith("/") ? normalized : `${normalized}/`),
    prefix: normalized.endsWith("/**") ? normalized.slice(0, -3) : undefined
  };
}

function isIgnored(path: string, isDirectory: boolean, ignorePatterns: CompiledIgnorePattern[]): boolean {
  return ignorePatterns.some((pattern) => {
    if (pattern.prefix) {
      const prefix = pattern.prefix;
      return path === prefix || path.startsWith(`${prefix}/`);
    }
    return pattern.regex.test(path) ||
      (isDirectory && pattern.directoryRegex.test(`${path}/`)) ||
      path === pattern.normalized ||
      path.startsWith(`${pattern.normalized.replace(/\/$/, "")}/`);
  });
}

async function summarizeContent(path: string, bytes: number): Promise<Omit<FileSummary, "path" | "bytes">> {
  if (bytes > 1_000_000) return {};

  try {
    const content = await readFile(path, "utf8");
    return {
      lines: content.length === 0 ? 0 : content.split("\n").length,
      imports: extractImports(content, path),
      hasUseClient: /^["']use client["'];?/m.test(content),
      envAccesses: extractEnvAccesses(content, path),
      hasDirectDbAccess: hasDirectDbAccess(content)
    };
  } catch {
    return {};
  }
}

function extractImports(content: string, path: string): string[] {
  if (/\.(ts|tsx|js|jsx|mts|cts|mjs|cjs)$/.test(path)) {
    return extractImportsWithTypescript(content, path);
  }

  const imports = new Set<string>();
  const patterns = [
    /import\s+(?:type\s+)?(?:[^'"]+\s+from\s+)?["']([^"']+)["']/g,
    /export\s+[^'"]+\s+from\s+["']([^"']+)["']/g,
    /require\(["']([^"']+)["']\)/g
  ];

  for (const pattern of patterns) {
    for (const match of content.matchAll(pattern)) {
      if (match[1]) imports.add(match[1]);
    }
  }

  return [...imports].sort();
}

function extractEnvAccesses(content: string, path: string): string[] {
  if (/\.(ts|tsx|js|jsx|mts|cts|mjs|cjs)$/.test(path)) {
    return extractEnvAccessesWithTypescript(content, path);
  }

  const envAccesses = new Set<string>();
  for (const match of content.matchAll(/process\.env\.([A-Z0-9_]+)/g)) {
    if (match[1]) envAccesses.add(match[1]);
  }
  return [...envAccesses].sort();
}

function hasDirectDbAccess(content: string): boolean {
  return /\b(sql`|db\.(select|insert|update|delete|query)|createClient\(|drizzle\(|prisma\.)/.test(content);
}

function extractImportsWithTypescript(content: string, path: string): string[] {
  const sourceFile = ts.createSourceFile(path, content, ts.ScriptTarget.Latest, true);
  const imports = new Set<string>();

  sourceFile.forEachChild((node) => {
    if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) && node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier)) {
      imports.add(node.moduleSpecifier.text);
    }

    if (ts.isImportEqualsDeclaration(node) && ts.isExternalModuleReference(node.moduleReference) && ts.isStringLiteral(node.moduleReference.expression)) {
      imports.add(node.moduleReference.expression.text);
    }
  });

  return [...imports].sort();
}

function extractEnvAccessesWithTypescript(content: string, path: string): string[] {
  const sourceFile = ts.createSourceFile(path, content, ts.ScriptTarget.Latest, true);
  const envAccesses = new Set<string>();

  function visit(node: ts.Node): void {
    if (
      ts.isPropertyAccessExpression(node) &&
      ts.isPropertyAccessExpression(node.expression) &&
      ts.isIdentifier(node.expression.expression) &&
      node.expression.expression.text === "process" &&
      node.expression.name.text === "env"
    ) {
      envAccesses.add(node.name.text);
    }

    if (
      ts.isElementAccessExpression(node) &&
      ts.isPropertyAccessExpression(node.expression) &&
      ts.isIdentifier(node.expression.expression) &&
      node.expression.expression.text === "process" &&
      node.expression.name.text === "env" &&
      ts.isStringLiteralLike(node.argumentExpression)
    ) {
      envAccesses.add(node.argumentExpression.text);
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return [...envAccesses].sort();
}
