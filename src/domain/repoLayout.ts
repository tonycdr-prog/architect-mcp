import type { ArchitectureContract, DirectoryRule, FileRule, RepoLayout } from "./types.js";
import { normalizePath } from "./pathRules.js";

export function applyRepoLayoutToContract(contract: ArchitectureContract, layout?: RepoLayout): ArchitectureContract {
  if (!layout || Object.keys(layout.pathMap).length === 0) return contract;

  return {
    ...contract,
    directories: dedupeDirectories(contract.directories.flatMap((directory) => mapDirectory(directory, layout))),
    fileRules: contract.fileRules.map((rule) => mapFileRule(rule, layout)),
    agentInstructions: [
      ...contract.agentInstructions,
      "Use the repo layout mapping in this contract when interpreting canonical paths from stack packs."
    ]
  };
}

export function inferRepoLayoutFromFiles(paths: string[]): RepoLayout {
  const normalizedPaths = paths.map(normalizePath);
  const hasPrefix = (prefix: string) => normalizedPaths.some((path) => path === prefix || path.startsWith(`${prefix}/`));

  const pathMap: Record<string, string[]> = {};

  if (hasPrefix("client")) {
    pathMap["src/features"] = ["client"];
    pathMap["src/shared/ui"] = ["client/components"];
  }

  if (hasPrefix("admin/react")) {
    pathMap["src/app"] = ["admin/react"];
    pathMap["src/features"] = [...(pathMap["src/features"] ?? []), "admin/react/pages"];
    pathMap["src/shared/ui"] = [...(pathMap["src/shared/ui"] ?? []), "admin/react/components"];
  }

  if (hasPrefix("shared")) {
    pathMap["src/shared"] = ["shared"];
    pathMap["src/db"] = ["shared"];
    pathMap["src/db/schema"] = ["shared"];
  }

  if (hasPrefix("server")) {
    pathMap["src/server"] = ["server"];
    pathMap["src/server/routes"] = ["server/routes"];
    pathMap["src/server/services"] = ["server/services", "server/lib"];
    pathMap["src/server/adapters"] = ["server/lib", "server/services"];
    pathMap["src/db/migrations"] = ["server/migrations"];
    pathMap["src/db/repositories"] = ["server/services", "server/lib"];
  }

  if (hasPrefix("supabase")) {
    pathMap["src/db"] = [...(pathMap["src/db"] ?? []), "supabase"];
    pathMap["src/db/supabase"] = ["supabase"];
  }

  if (normalizedPaths.some((path) => path.includes("/__tests__/") || path.includes(".test."))) {
    pathMap.tests = ["client", "server", "admin"];
  }

  return { pathMap };
}

function mapDirectory(directory: DirectoryRule, layout: RepoLayout): DirectoryRule[] {
  const mappedPaths = mapPath(directory.path, layout);
  return mappedPaths.map((path) => ({
    ...directory,
    path
  }));
}

function mapFileRule(rule: FileRule, layout: RepoLayout): FileRule {
  return {
    ...rule,
    appliesToPaths: rule.appliesToPaths?.flatMap((pathPattern) => mapPath(pathPattern, layout))
  };
}

function mapPath(path: string, layout: RepoLayout): string[] {
  const normalizedPath = normalizePath(path);
  const entries = Object.entries(layout.pathMap)
    .map(([canonical, replacements]) => ({
      canonical: normalizePath(canonical),
      replacements: replacements.map(normalizePath)
    }))
    .sort((left, right) => right.canonical.length - left.canonical.length);

  for (const entry of entries) {
    if (normalizedPath === entry.canonical) return entry.replacements;
    if (normalizedPath.startsWith(`${entry.canonical}/`)) {
      const suffix = normalizedPath.slice(entry.canonical.length);
      return entry.replacements.map((replacement) => `${replacement}${suffix}`);
    }
  }

  return [normalizedPath];
}

function dedupeDirectories(directories: DirectoryRule[]): DirectoryRule[] {
  const byPath = new Map<string, DirectoryRule>();

  for (const directory of directories) {
    const existing = byPath.get(directory.path);
    if (!existing) {
      byPath.set(directory.path, directory);
      continue;
    }

    byPath.set(directory.path, {
      ...existing,
      required: existing.required || directory.required,
      purpose: existing.purpose === directory.purpose ? existing.purpose : `${existing.purpose} ${directory.purpose}`
    });
  }

  return [...byPath.values()];
}
