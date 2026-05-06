import { createFinding } from "./findingMetadata.js";
import { isGeneratedFile, normalizePath } from "./pathRules.js";
import { dedupeViolations, isUiFile } from "./reviewerPredicates.js";
import type { FileSummary, ReviewViolation } from "./types.js";

export function reviewImportDirection(files: Array<FileSummary & { path: string }>): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  for (const file of files) {
    if (isGeneratedFile(file.path) || !file.path.startsWith("src/shared/")) continue;
    const featureImport = (file.imports ?? []).find((specifier) =>
      specifier.includes("/features/") ||
      specifier.startsWith("../features") ||
      specifier.startsWith("../../features") ||
      specifier.startsWith("@/features/")
    );
    if (featureImport) {
      violations.push(createFinding({
        code: "ARCH007_SHARED_IMPORTS_FEATURE",
        severity: "error",
        path: file.path,
        message: `Shared module imports feature code: ${featureImport}.`,
        recommendation: "Invert the dependency. Feature modules may import shared code, but shared code must not import feature modules."
      }));
    }
  }
  return violations;
}

export function reviewImportGraph(files: Array<FileSummary & { path: string }>): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  const fileMap = new Map(files.map((file) => [file.path, file]));
  const graph = new Map<string, string[]>();
  for (const file of files) {
    graph.set(file.path, (file.imports ?? [])
      .map((specifier) => resolveImportPath(file.path, specifier, fileMap))
      .filter((path): path is string => Boolean(path)));
  }

  for (const file of files) {
    if (isUiFile(file.path)) {
      const dbPath = reachablePath(file.path, graph, (path) => /^src\/db\//.test(path) || path.includes("/db/"));
      if (dbPath) {
        violations.push(createFinding({
          code: "ARCH022_IMPORT_GRAPH_BOUNDARY",
          severity: "error",
          path: file.path,
          message: `UI file reaches database code through imports: ${dbPath}.`,
          recommendation: "Route UI data access through server/client-safe APIs instead of transitive database imports."
        }));
      }
    }

    if (file.path.startsWith("src/shared/")) {
      const featurePath = reachablePath(file.path, graph, (path) => path.startsWith("src/features/"));
      if (featurePath) {
        violations.push(createFinding({
          code: "ARCH022_IMPORT_GRAPH_BOUNDARY",
          severity: "error",
          path: file.path,
          message: `Shared module reaches feature code through imports: ${featurePath}.`,
          recommendation: "Invert the dependency so features import shared modules, not the reverse."
        }));
      }
    }

    if (/\/routes?\//.test(file.path)) {
      const imports = graph.get(file.path) ?? [];
      if (imports.length > 0 && file.lines && file.lines > 120 && !imports.some((path) => /\/(services|use-cases)\//.test(path))) {
        violations.push(createFinding({
          code: "ARCH022_IMPORT_GRAPH_BOUNDARY",
          severity: "warning",
          path: file.path,
          message: "Route file is substantial but does not import a service/use-case module.",
          recommendation: "Keep routes thin and move workflow orchestration into services or use-cases."
        }));
      }
    }

    const cycle = reachablePath(file.path, graph, (path) => path === file.path, new Set(), true);
    if (cycle) {
      violations.push(createFinding({
        code: "ARCH022_IMPORT_GRAPH_BOUNDARY",
        severity: "warning",
        path: file.path,
        message: "Import graph contains a circular dependency.",
        recommendation: "Extract shared contracts or invert one dependency to remove the cycle."
      }));
    }
  }

  return dedupeViolations(violations);
}

function reachablePath(start: string, graph: Map<string, string[]>, predicate: (path: string) => boolean, seen = new Set<string>(), skipStartPredicate = false): string | undefined {
  if (seen.has(start)) return undefined;
  seen.add(start);
  if (!skipStartPredicate && predicate(start)) return start;
  for (const next of graph.get(start) ?? []) {
    if (predicate(next)) return next;
    const found = reachablePath(next, graph, predicate, seen);
    if (found) return found;
  }
  return undefined;
}

function resolveImportPath(fromPath: string, specifier: string, fileMap: Map<string, FileSummary>): string | undefined {
  if (!specifier.startsWith(".") && !specifier.startsWith("@/")) return undefined;
  const base = specifier.startsWith("@/") ? `src/${specifier.slice(2)}` : normalizeRelativeImport(fromPath, specifier);
  const candidates = [base, `${base}.ts`, `${base}.tsx`, `${base}.js`, `${base}.jsx`, `${base}/index.ts`, `${base}/index.tsx`];
  return candidates.find((candidate) => fileMap.has(candidate));
}

function normalizeRelativeImport(fromPath: string, specifier: string): string {
  const parts = fromPath.split("/").slice(0, -1);
  for (const part of specifier.split("/")) {
    if (!part || part === ".") continue;
    if (part === "..") parts.pop();
    else parts.push(part);
  }
  return normalizePath(parts.join("/"));
}
