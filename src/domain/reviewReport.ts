import type { ReviewMode, ReviewOptions, ReviewReport, ReviewViolation } from "./types.js";
import { matchesPathPattern, normalizePath } from "./pathRules.js";

const DEFAULT_IGNORE_PATTERNS = [
  "docs/audits/*.json",
  "docs/audits/**/*.json",
  "**/__tests__/**",
  "**/*.test.ts",
  "**/*.test.tsx",
  "**/*.test.js",
  "**/*.test.jsx",
  "__mocks__/**",
  "config/**/*.json"
];

export function createReviewReport(violations: ReviewViolation[], options: ReviewOptions = {}): ReviewReport {
  const mode = options.mode ?? "summary";
  const ignorePatterns = [...DEFAULT_IGNORE_PATTERNS, ...(options.ignorePatterns ?? [])];
  const summarizeLineWarningsBelow = options.summarizeLineWarningsBelow ?? (mode === "strict" ? 0 : 800);
  const maxDetailedFindings = options.maxDetailedFindings ?? defaultFindingLimit(mode);

  const ignored = violations.filter((violation) => violation.path && ignorePatterns.some((pattern) => matchesPathPattern(violation.path ?? "", pattern)));
  const baselineFindings = violations.filter((violation) => isInBaseline(violation, options.baseline?.findings ?? []));
  const eligible = violations.filter((violation) => !ignored.includes(violation) && !baselineFindings.includes(violation));
  const groupedLineWarnings = eligible.filter((violation) => isLowPriorityLineWarning(violation, summarizeLineWarningsBelow));
  const candidates = eligible.filter((violation) => !groupedLineWarnings.includes(violation));
  const modeFiltered = filterByMode(candidates, mode);
  const priorityFindings = sortFindings(modeFiltered).slice(0, maxDetailedFindings);
  const groups = groupViolations([...groupedLineWarnings, ...modeFiltered]);
  const shownSet = new Set(priorityFindings);
  const violationsForMode = mode === "strict" ? eligible : priorityFindings;
  const errors = eligible.filter((violation) => violation.severity === "error").length;
  const warnings = eligible.filter((violation) => violation.severity === "warning").length;
  const scoreEligible = violations.filter((violation) => !baselineFindings.includes(violation));
  const scoreErrors = scoreEligible.filter((violation) => violation.severity === "error").length;
  const scoreWarnings = scoreEligible.filter((violation) => violation.severity === "warning").length;
  const suppressed = Math.max(0, eligible.length - shownSet.size);
  const noiseSuppressed = ignored.length + groupedLineWarnings.length;
  const baselineSuppressed = baselineFindings.length;
  const score = scoreReview(scoreErrors, scoreWarnings);
  const lifecycleGate = {
    acceptedWithoutReason: countAcceptedWithoutReason(options.baseline),
    newHighConfidenceErrors: eligible.filter((violation) => violation.severity === "error" && violation.confidence === "high").length
  };

  return {
    score,
    grade: gradeReview(score),
    mode,
    gate: createReviewGate(errors, warnings, score, mode, options.gate, lifecycleGate),
    summary: {
      errors,
      warnings,
      shown: violationsForMode.length,
      suppressed,
      noiseSuppressed,
      baselineSuppressed
    },
    groups,
    priorityFindings,
    violations: violationsForMode
  };
}

function defaultFindingLimit(mode: ReviewMode): number {
  if (mode === "ci") return 20;
  if (mode === "migration") return 30;
  if (mode === "summary") return 40;
  return Number.MAX_SAFE_INTEGER;
}

function filterByMode(violations: ReviewViolation[], mode: ReviewMode): ReviewViolation[] {
  if (mode === "ci") return violations.filter((violation) => violation.severity === "error");
  if (mode === "migration") return violations.filter((violation) => violation.severity === "error" || !isLineWarning(violation));
  return violations;
}

function groupViolations(violations: ReviewViolation[]) {
  const groups = new Map<string, ReviewViolation[]>();

  for (const violation of violations) {
    const key = groupKey(violation);
    groups.set(key, [...(groups.get(key) ?? []), violation]);
  }

  return [...groups.entries()]
    .map(([key, grouped]) => ({
      key,
      count: grouped.length,
      severity: grouped.some((violation) => violation.severity === "error") ? "error" as const : "warning" as const,
      samplePaths: grouped.map((violation) => violation.path).filter(Boolean).slice(0, 8) as string[],
      recommendation: grouped[0]?.recommendation ?? "Review this group."
    }))
    .sort((left, right) => right.count - left.count);
}

function groupKey(violation: ReviewViolation): string {
  if (isLineWarning(violation)) return `oversized-files:${topDirectory(violation.path)}`;
  if (violation.message.startsWith("Required architecture directory")) return "missing-required-directories";
  return violation.code;
}

function topDirectory(path?: string): string {
  return normalizePath(path ?? "(root)").split("/")[0] ?? "(root)";
}

function sortFindings(violations: ReviewViolation[]): ReviewViolation[] {
  return [...violations].sort((left, right) => severityWeight(right) - severityWeight(left) || lineCount(right) - lineCount(left));
}

function severityWeight(violation: ReviewViolation): number {
  return violation.severity === "error" ? 2 : 1;
}

function isLowPriorityLineWarning(violation: ReviewViolation, threshold: number): boolean {
  return isLineWarning(violation) && lineCount(violation) < threshold;
}

function isLineWarning(violation: ReviewViolation): boolean {
  return violation.code === "ARCH001_OVERSIZED_FILE";
}

function lineCount(violation: ReviewViolation): number {
  return Number(violation.message.match(/^File has (\d+) lines/)?.[1] ?? 0);
}

function isInBaseline(violation: ReviewViolation, baseline: NonNullable<ReviewOptions["baseline"]>["findings"]): boolean {
  return baseline.some((finding) => {
    if (finding.code !== violation.code) return false;
    if (!finding.path && !finding.message) return false;
    if (finding.path && finding.path !== violation.path) return false;
    if (finding.message && finding.message !== violation.message) return false;
    return true;
  });
}

function createReviewGate(
  errors: number,
  warnings: number,
  score: number,
  mode: ReviewMode,
  gateOptions: ReviewOptions["gate"],
  lifecycle: ReviewReport["gate"]["lifecycle"]
): ReviewReport["gate"] {
  const thresholds = {
    maxErrors: gateOptions?.maxErrors ?? 0,
    maxWarnings: gateOptions?.maxWarnings ?? defaultMaxWarnings(mode),
    minScore: gateOptions?.minScore ?? defaultMinScore(mode)
  };

  if (errors > thresholds.maxErrors) {
    return {
      status: "fail",
      thresholds,
      lifecycle,
      reason: `Found ${errors} error finding${errors === 1 ? "" : "s"}; maximum allowed is ${thresholds.maxErrors}.`
    };
  }

  if (lifecycle && lifecycle.acceptedWithoutReason > 0) {
    return {
      status: "fail",
      thresholds,
      lifecycle,
      reason: `Found ${lifecycle.acceptedWithoutReason} accepted baseline finding${lifecycle.acceptedWithoutReason === 1 ? "" : "s"} without a reason.`
    };
  }

  if (warnings > thresholds.maxWarnings) {
    return {
      status: "warn",
      thresholds,
      lifecycle,
      reason: `Found ${warnings} warning finding${warnings === 1 ? "" : "s"}; warning threshold is ${thresholds.maxWarnings}.`
    };
  }

  if (score < thresholds.minScore) {
    return {
      status: "warn",
      thresholds,
      lifecycle,
      reason: `Architecture score ${score.toFixed(1)} is below the ${thresholds.minScore} target.`
    };
  }

  return {
    status: "pass",
    thresholds,
    lifecycle,
    reason: "No review findings exceeded the configured gate."
  };
}

function countAcceptedWithoutReason(baseline: ReviewOptions["baseline"]): number {
  return baseline?.findings.filter((finding) => finding.status === "accepted" && !finding.reason?.trim()).length ?? 0;
}

function defaultMaxWarnings(mode: ReviewMode): number {
  if (mode === "ci") return 0;
  if (mode === "strict") return 20;
  if (mode === "migration") return 200;
  return 50;
}

function defaultMinScore(mode: ReviewMode): number {
  if (mode === "ci") return 90;
  if (mode === "strict") return 80;
  if (mode === "migration") return 60;
  return 70;
}

function scoreReview(errors: number, warnings: number): number {
  return Math.max(0, Math.min(100, 100 - errors * 12 - Math.min(warnings, 80) * 0.35));
}

function gradeReview(score: number): ReviewReport["grade"] {
  if (score >= 90) return "A";
  if (score >= 80) return "B";
  if (score >= 70) return "C";
  if (score >= 60) return "D";
  return "F";
}
