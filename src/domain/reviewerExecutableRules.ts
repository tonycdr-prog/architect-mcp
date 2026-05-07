import { createFinding } from "./findingMetadata.js";
import { isGeneratedFile, isSourceCodeFile, matchesPathPattern, normalizePath } from "./pathRules.js";
import { dedupeViolations, importsServerOnlyModule, isUiFile } from "./reviewerPredicates.js";
import { inferTriggerKind } from "./stackPacks.js";
import type { ArchitectureContract, FileRule, FileSummary, ReviewViolation, RuleTriggerKind } from "./types.js";

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
      if (!fileTriggersRule(file, rule)) continue;

      violations.push(createFinding({
        code: codeForPackRule(rule.name, triggerKindsForRule(rule)[0]),
        severity: rule.severity,
        path: file.path,
        message: `${rule.name}: ${rule.rule}`,
        recommendation: rule.recommendation ?? "Follow the architecture contract rule for this file."
      }));
    }
  }

  return dedupeViolations(violations);
}

function codeForPackRule(ruleName: string, triggerKind = inferTriggerKind(ruleName)): ReviewViolation["code"] {
  if (triggerKind === "route-thinness") return "ARCH004_THIN_CONTROLLER";
  if (triggerKind === "import-boundary") return "ARCH011_TRANSPORT_LEAK";
  if (triggerKind === "direct-db-access") return "ARCH002_UI_DB_ACCESS";
  if (triggerKind === "env-access") return "ARCH008_ENV_SCATTER";
  if (triggerKind === "migration-discipline") return "ARCH012_MIGRATION_DISCIPLINE";
  if (triggerKind === "client-boundary") return /supabase/i.test(ruleName) ? "ARCH013_SUPABASE_SPLIT" : "ARCH003_CLIENT_SERVER_LEAK";
  return "ARCH014_PACK_RULE";
}

function fileTriggersRule(file: FileSummary, rule: FileRule): boolean {
  const triggerKinds = triggerKindsForRule(rule);

  if (triggerKinds.includes("route-thinness")) return Boolean(file.lines && file.lines > 180);
  if (triggerKinds.includes("import-boundary")) return hasTransportLeak(file);
  if (triggerKinds.includes("line-threshold")) return Boolean(isUiFile(file.path) && file.lines && file.lines > 220);
  if (triggerKinds.includes("client-boundary")) return Boolean((file.hasUseClient && /\/(app|pages)\//.test(normalizePath(file.path))) || (file.hasUseClient && importsServerOnlyModule(file.imports ?? [])));
  if (triggerKinds.includes("direct-db-access")) return Boolean(file.hasDirectDbAccess);
  if (triggerKinds.includes("env-access")) return Boolean(file.envAccesses?.length && !isConfigEnvModule(file.path));
  if (triggerKinds.includes("migration-discipline")) return Boolean(normalizePath(file.path).includes("src/db/schema/"));
  if (triggerKinds.includes("hosted-filesystem")) return Boolean((file.imports ?? []).some((specifier) => specifier.includes("scanWorkspace")));
  if (triggerKinds.includes("auth-boundary")) return hasAuthBoundaryLeak(file);
  if (triggerKinds.includes("payment-boundary")) return hasPaymentBoundaryLeak(file);
  if (triggerKinds.includes("test-policy")) return needsTestCoverage(file);
  if (triggerKinds.includes("validation-boundary")) return hasValidationBoundaryLeak(file);
  if (triggerKinds.includes("ai-tool-safety")) return hasAiToolSafetyLeak(file);
  return false;
}

function triggerKindsForRule(rule: FileRule): RuleTriggerKind[] {
  const kinds = [
    rule.triggerKind,
    ...(rule.detectors?.map((detector) => detector.kind) ?? [])
  ].filter((kind): kind is RuleTriggerKind => Boolean(kind));
  return kinds.length ? [...new Set(kinds)] : [inferTriggerKind(rule.name)];
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
