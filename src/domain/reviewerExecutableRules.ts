import { createFinding } from "./findingMetadata.js";
import { isGeneratedFile, isSourceCodeFile, matchesPathPattern, normalizePath } from "./pathRules.js";
import { dedupeViolations, importsServerOnlyModule, isUiFile } from "./reviewerPredicates.js";
import type { ArchitectureContract, FileSummary, ReviewViolation } from "./types.js";

export function reviewExecutableFileRules(
  files: Array<FileSummary & { path: string }>,
  contract?: ArchitectureContract
): ReviewViolation[] {
  const violations: ReviewViolation[] = [];

  for (const rule of contract?.fileRules ?? []) {
    const matchingFiles = files.filter((file) =>
      !isGeneratedFile(file.path) &&
      rule.appliesToPaths?.some((pattern) => matchesPathPattern(file.path, pattern))
    );

    for (const file of matchingFiles) {
      if (!fileTriggersRule(file, rule.name)) continue;

      violations.push(createFinding({
        code: codeForPackRule(rule.name),
        severity: rule.severity,
        path: file.path,
        message: `${rule.name}: ${rule.rule}`,
        recommendation: rule.recommendation ?? "Follow the architecture contract rule for this file."
      }));
    }
  }

  return dedupeViolations(violations);
}

function codeForPackRule(ruleName: string): ReviewViolation["code"] {
  const normalizedRuleName = ruleName.toLowerCase();
  if (normalizedRuleName.includes("thin controller") || normalizedRuleName.includes("thin route")) return "ARCH004_THIN_CONTROLLER";
  if (normalizedRuleName.includes("transport-neutral tools")) return "ARCH011_TRANSPORT_LEAK";
  if (normalizedRuleName.includes("ui queries")) return "ARCH002_UI_DB_ACCESS";
  if (normalizedRuleName.includes("env")) return "ARCH008_ENV_SCATTER";
  if (normalizedRuleName.includes("migration discipline")) return "ARCH012_MIGRATION_DISCIPLINE";
  if (normalizedRuleName.includes("supabase split")) return "ARCH013_SUPABASE_SPLIT";
  return "ARCH014_PACK_RULE";
}

function fileTriggersRule(file: FileSummary, ruleName: string): boolean {
  const normalizedRuleName = ruleName.toLowerCase();

  if (normalizedRuleName.includes("thin route") || normalizedRuleName.includes("thin controller")) return Boolean(file.lines && file.lines > 180);
  if (normalizedRuleName.includes("transport-neutral tools")) return hasTransportLeak(file);
  if (normalizedRuleName.includes("god component")) return Boolean(isUiFile(file.path) && file.lines && file.lines > 220);
  if (normalizedRuleName.includes("client boundaries")) return Boolean(file.hasUseClient && /\/(app|pages)\//.test(normalizePath(file.path)));
  if (normalizedRuleName.includes("ui queries")) return Boolean(file.hasDirectDbAccess);
  if (normalizedRuleName.includes("env")) return Boolean(file.envAccesses?.length && !isConfigEnvModule(file.path));
  if (normalizedRuleName.includes("migration discipline")) return Boolean(normalizePath(file.path).includes("src/db/schema/"));
  if (normalizedRuleName.includes("supabase split")) return Boolean(file.hasUseClient && importsServerOnlyModule(file.imports ?? []));
  if (normalizedRuleName.includes("hosted filesystem scanning")) return Boolean((file.imports ?? []).some((specifier) => specifier.includes("scanWorkspace")));
  if (normalizedRuleName.includes("auth")) return hasAuthBoundaryLeak(file);
  if (normalizedRuleName.includes("payment") || normalizedRuleName.includes("stripe")) return hasPaymentBoundaryLeak(file);
  if (normalizedRuleName.includes("test") || normalizedRuleName.includes("vitest")) return needsTestCoverage(file);
  if (normalizedRuleName.includes("validation") || normalizedRuleName.includes("zod")) return hasValidationBoundaryLeak(file);
  if (normalizedRuleName.includes("ai tool") || normalizedRuleName.includes("model call")) return hasAiToolSafetyLeak(file);
  return false;
}

function hasTransportLeak(file: FileSummary): boolean {
  return Boolean(
    normalizePath(file.path).startsWith("src/tools/") &&
      (file.imports ?? []).some((specifier) =>
        specifier.includes("stdio") ||
        specifier.includes("streamableHttp") ||
        specifier.includes("express") ||
        specifier.includes("scanWorkspace")
      )
  );
}

function hasAuthBoundaryLeak(file: FileSummary): boolean {
  return Boolean(
    isUiFile(file.path) &&
      ((file.imports ?? []).some((specifier) => /auth|session|jwt|cookie/i.test(specifier)) ||
        file.envAccesses?.some((env) => /AUTH|JWT|SESSION|SECRET/i.test(env)))
  );
}

function hasPaymentBoundaryLeak(file: FileSummary): boolean {
  const normalizedPath = normalizePath(file.path);
  const isServerPaymentBoundary = /\/server\/(payments|billing)\//.test(normalizedPath) || isConfigEnvModule(file.path);

  return Boolean(
    (isUiFile(file.path) && (file.imports ?? []).some((specifier) => /stripe|checkout|payment/i.test(specifier))) ||
      (!isServerPaymentBoundary && file.envAccesses?.some((env) => /STRIPE|PAYMENT|WEBHOOK/i.test(env)))
  );
}

function isConfigEnvModule(path: string): boolean {
  return /(^|\/)(config|env)\.ts$/.test(normalizePath(path));
}

function needsTestCoverage(file: FileSummary): boolean {
  return Boolean(isSourceCodeFile(file.path) && !/(\.test\.|\/tests?\/|\/__tests__\/)/.test(normalizePath(file.path)) && file.lines && file.lines > 220);
}

function hasValidationBoundaryLeak(file: FileSummary): boolean {
  return Boolean((file.imports ?? []).some((specifier) => /zod|valibot|yup/i.test(specifier)) && /\/(components|pages|screens|app)\//.test(normalizePath(file.path)));
}

function hasAiToolSafetyLeak(file: FileSummary): boolean {
  return Boolean(
    (file.imports ?? []).some((specifier) => /openai|ai\/|langchain|anthropic/i.test(specifier)) &&
      !/\/(server|services|tools|ai)\//.test(normalizePath(file.path))
  );
}
