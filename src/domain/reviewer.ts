import type { ArchitectureContract, BuildPlan, FileSummary, ReviewViolation } from "./types.js";
import { createFinding, lineThresholdForPath } from "./findingMetadata.js";
import { isGeneratedFile, isSourceCodeFile, normalizePath } from "./pathRules.js";
import { reviewImportDirection, reviewImportGraph } from "./reviewerImportGraph.js";
import { reviewExecutableFileRules } from "./reviewerExecutableRules.js";
import { reviewImplementationPlanDrift } from "./reviewerPlanDrift.js";
import { dedupeViolations, importsServerOnlyModule, isUiFile } from "./reviewerPredicates.js";

const DEFAULT_MAX_LINES = 300;
const TYPE_AGGREGATION_MAX_LINES = 300;
const SCHEMA_AGGREGATION_MAX_LINES = 300;
const ALLOWED_ROOT_SOURCE_FILES = [
  /^eslint\.config\.(js|mjs|cjs|ts)$/,
  /^prettier\.config\.(js|mjs|cjs|ts)$/,
  /^vite\.config\.(ts|js|mts|mjs)$/,
  /^vitest\.config\.(ts|js|mts|mjs)$/,
  /^next\.config\.(ts|js|mjs)$/,
  /^tailwind\.config\.(ts|js|cjs|mjs)$/,
  /^postcss\.config\.(js|cjs|mjs)$/,
  /^playwright\.config\.(ts|js)$/,
  /^tsup\.config\.(ts|js)$/,
  /^.*\.d\.ts$/
];
const MIXED_CONCERN_PATTERNS = [
  { pattern: /components?\/.*((^|[/. _-])db([/. _-]|$)|database|migration|schema|repository)/i, concern: "database logic inside component paths" },
  { pattern: /(page|route|component)\.(tsx|jsx)$.*(service|repository)/i, concern: "service/repository naming mixed into UI files" }
];

export function reviewFileSummaries(
  files: FileSummary[],
  contract?: ArchitectureContract,
  maxLines = DEFAULT_MAX_LINES,
  directories: string[] = [],
  buildPlan?: BuildPlan
): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  const filePaths = [
    ...files.map((file) => normalizePath(file.path)),
    ...directories.map(normalizePath)
  ];
  const normalizedFiles = files.map((file) => ({
    ...file,
    path: normalizePath(file.path)
  }));

  for (const file of normalizedFiles) {
    const path = file.path;
    if (isGeneratedFile(path)) continue;

    const threshold = lineThresholdForPath(path, maxLines);
    if (file.lines !== undefined && file.lines > threshold.maxLines) {
      violations.push(createFinding({
        code: "ARCH001_OVERSIZED_FILE",
        severity: "warning",
        path,
        message: `File has ${file.lines} lines, above the ${threshold.maxLines}-line ${threshold.category} threshold.`,
        recommendation: "Split by responsibility or add this file to the baseline if the size is intentional."
      }));
    }

    for (const mixedConcern of MIXED_CONCERN_PATTERNS) {
      if (mixedConcern.pattern.test(path)) {
        violations.push(createFinding({
          code: "ARCH006_MIXED_CONCERNS",
          severity: "error",
          path,
          message: `Possible mixed concern: ${mixedConcern.concern}.`,
          recommendation: "Move data access and infrastructure code behind a server/service boundary."
        }));
      }
    }

    if (isUiFile(path) && file.hasDirectDbAccess) {
      violations.push(createFinding({
        code: "ARCH002_UI_DB_ACCESS",
        severity: "error",
        path,
        message: "UI file appears to perform direct database access.",
        recommendation: "Move database access behind a server-only repository/service boundary and pass data into UI components."
      }));
    }

    if (isUiFile(path) && file.envAccesses?.some((envName) => !isPublicClientEnv(envName))) {
      violations.push(createFinding({
        code: "ARCH009_SERVER_ENV_IN_UI",
        severity: "error",
        path,
        message: "UI file reads server-only environment variables.",
        recommendation: "Move secret access into server-only code. Client-visible variables must use an explicit public prefix."
      }));
    }

    if (file.hasUseClient && importsServerOnlyModule(file.imports ?? [])) {
      violations.push(createFinding({
        code: "ARCH003_CLIENT_SERVER_LEAK",
        severity: "error",
        path,
        message: "Client component imports a server-only module.",
        recommendation: "Move the server dependency behind an API/server action or pass data into the client component."
      }));
    }
  }

  for (const directory of contract?.directories ?? []) {
    if (!directory.required) continue;
    const hasDirectory = filePaths.some((path) => path === directory.path || path.startsWith(`${directory.path}/`));
    if (!hasDirectory) {
      violations.push(createFinding({
        code: "ARCH005_MISSING_DIRECTORY",
        severity: "warning",
        path: directory.path,
        message: `Required architecture directory is missing: ${directory.path}.`,
        recommendation: `Create ${directory.path} for: ${directory.purpose}`
      }));
    }
  }

  if (filePaths.some((path) => /^src\/(app|pages)\/.*\.(tsx|jsx)$/.test(path)) && !filePaths.some((path) => path.startsWith("src/features/"))) {
    violations.push(createFinding({
      code: "ARCH010_ROUTE_WITHOUT_FEATURES",
      severity: "warning",
      message: "Routes/pages exist but no feature modules were found.",
      recommendation: "Move workflow-specific UI and logic into src/features/<feature-name>."
    }));
  }

  violations.push(...reviewImportDirection(normalizedFiles));
  violations.push(...reviewImportGraph(normalizedFiles));
  violations.push(...reviewEnvAccessPatterns(normalizedFiles));
  violations.push(...reviewRootHygiene(normalizedFiles));
  violations.push(...reviewTypeAndSchemaAggregation(normalizedFiles));
  violations.push(...reviewExecutableFileRules(normalizedFiles, contract));
  violations.push(...reviewImplementationPlanDrift(normalizedFiles, filePaths, buildPlan));

  return dedupeViolations(violations);
}

function reviewRootHygiene(files: Array<FileSummary & { path: string }>): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  for (const file of files) {
    if (isGeneratedFile(file.path) || !isSourceCodeFile(file.path) || file.path.includes("/")) continue;
    if (ALLOWED_ROOT_SOURCE_FILES.some((pattern) => pattern.test(file.path))) continue;

    violations.push(createFinding({
      code: "ARCH026_REPO_HYGIENE",
      severity: "warning",
      path: file.path,
      message: "Source file lives at the repository root.",
      recommendation: "Move product source into an owned directory such as src/domain, src/server, src/tools, or scripts; keep the root for package, config, and documentation files."
    }));
  }
  return violations;
}

function reviewTypeAndSchemaAggregation(files: Array<FileSummary & { path: string }>): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  for (const file of files) {
    if (file.lines === undefined || isGeneratedFile(file.path)) continue;
    const typeBucket = /(^|\/)(types|.*Types)\.ts$/.test(file.path);
    const schemaBucket = /(^|\/)schemas\.ts$/.test(file.path);
    const limit = typeBucket ? TYPE_AGGREGATION_MAX_LINES : schemaBucket ? SCHEMA_AGGREGATION_MAX_LINES : undefined;
    if (!limit || file.lines <= limit) continue;

    violations.push(createFinding({
      code: "ARCH025_TYPE_SCHEMA_AGGREGATION",
      severity: "warning",
      path: file.path,
      message: `Type/schema aggregation file has ${file.lines} lines, above the ${limit}-line aggregation threshold.`,
      recommendation: "Split type/schema definitions by domain and keep the shared entry file as a small barrel export."
    }));
  }
  return violations;
}

function isPublicClientEnv(envName: string): boolean {
  return envName === "NODE_ENV" ||
    envName.startsWith("NEXT_PUBLIC_") ||
    envName.startsWith("VITE_") ||
    envName.startsWith("EXPO_PUBLIC_");
}

function reviewEnvAccessPatterns(files: Array<FileSummary & { path: string }>): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  const envReaders = files.filter((file) => !isGeneratedFile(file.path) && isSourceCodeFile(file.path) && file.envAccesses?.length);
  const allowedConfigReaders = envReaders.filter((file) => /(^|\/)(config|env)\.ts$/.test(file.path));
  const scatteredReaders = envReaders.filter((file) => !/(^|\/)(config|env)\.ts$/.test(file.path));

  if (envReaders.length > 1 && allowedConfigReaders.length === 0 && scatteredReaders.length > 1) {
    violations.push(createFinding({
      code: "ARCH008_ENV_SCATTER",
      severity: "warning",
      message: "Environment variables are read directly from multiple files.",
      recommendation: "Parse environment variables once in a typed config/env module and import validated config elsewhere."
    }));
  }

  return violations;
}
