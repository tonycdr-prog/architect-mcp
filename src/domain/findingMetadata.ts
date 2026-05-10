import type { FindingCode, ReviewViolation } from "./types.js";

export type FileCategory =
  | "ui"
  | "route"
  | "service"
  | "schema"
  | "test"
  | "script"
  | "docs-data"
  | "config"
  | "source";

export type FileThreshold = {
  category: FileCategory;
  maxLines: number;
};

const THRESHOLDS: FileThreshold[] = [
  { category: "ui", maxLines: 450 },
  { category: "route", maxLines: 350 },
  { category: "service", maxLines: 700 },
  { category: "schema", maxLines: 1400 },
  { category: "test", maxLines: 900 },
  { category: "script", maxLines: 900 },
  { category: "docs-data", maxLines: 2500 },
  { category: "config", maxLines: 800 },
  { category: "source", maxLines: 300 }
];

export function createFinding(input: Omit<ReviewViolation, "confidence"> & { confidence?: ReviewViolation["confidence"] }): ReviewViolation {
  return {
    ...input,
    confidence: input.confidence ?? defaultConfidence(input.code)
  };
}

export function categorizePath(path: string): FileCategory {
  if (/\/__tests__\/|(^|\/)(tests?|spec)\/|\.test\.(ts|tsx|js|jsx|py)$|_test\.go$|(^|\/)test_.*\.py$|(^|\/).*_test\.py$|(^|\/).*Tests?\.java$/.test(path)) return "test";
  if (/\.(md|mdx|rst|adoc|1)$/.test(path) || /^(README|CHANGELOG|CONTRIBUTING|AGENTS|CLAUDE)\.md$/.test(path) || /^docs\/.*\.(json|js|md)$/.test(path)) return "docs-data";
  if (/^(scripts|tools)\//.test(path)) return "script";
  if (/^(config\/|.*config\.(ts|js|json)$|.*\.config\.(ts|js)$)/.test(path) || /\.(ya?ml|toml|xml|jmx|properties)$/.test(path) || /(^|\/)pom\.xml$/.test(path)) return "config";
  if (/schema|types\.ts$/.test(path)) return "schema";
  if (/\/routes?\//.test(path)) return "route";
  if (/\/(services|lib)\//.test(path)) return "service";
  if (/\.(tsx|jsx)$/.test(path) || /\/(components|screens|pages)\//.test(path)) return "ui";
  return "source";
}

export function lineThresholdForPath(path: string, fallback: number): FileThreshold {
  const category = categorizePath(path);
  return THRESHOLDS.find((threshold) => threshold.category === category) ?? { category, maxLines: fallback };
}

export function findingIdentity(finding: Pick<ReviewViolation, "code" | "path" | "message">): string {
  return `${finding.code}:${finding.path ?? ""}:${finding.message}`;
}

function defaultConfidence(code: FindingCode): ReviewViolation["confidence"] {
  if (code === "ARCH002_UI_DB_ACCESS" || code === "ARCH003_CLIENT_SERVER_LEAK" || code === "ARCH009_SERVER_ENV_IN_UI") {
    return "high";
  }

  if (code === "ARCH001_OVERSIZED_FILE" || code === "ARCH004_THIN_CONTROLLER" || code === "ARCH005_MISSING_DIRECTORY" || code === "ARCH025_TYPE_SCHEMA_AGGREGATION" || code === "ARCH026_REPO_HYGIENE") {
    return "medium";
  }

  return "low";
}
