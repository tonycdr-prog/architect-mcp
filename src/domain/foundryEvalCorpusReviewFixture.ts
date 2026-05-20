import type {
  ReviewCoverage,
  ReviewFindingHistogramEntry,
  ReviewGate,
  ReviewGroup,
  ReviewReport,
  ReviewViolation
} from "./types.js";

export function foundryEvalReviewReport(input: {
  violations: ReviewViolation[];
  scanTruncated?: boolean;
  filesReviewed?: number;
  maxFiles?: number;
  caveats?: string[];
}): ReviewReport {
  const histogram = input.violations.reduce<ReviewFindingHistogramEntry[]>((entries, violation) => {
    const existing = entries.find((entry) => entry.code === violation.code && entry.severity === violation.severity);
    if (existing) existing.count += 1;
    else entries.push({ code: violation.code, severity: violation.severity, count: 1 });
    return entries;
  }, []);
  const coverage: ReviewCoverage = {
    totalFindings: input.violations.length,
    eligibleFindings: input.violations.length,
    detailedFindings: input.violations.length,
    suppressedFindings: 0,
    ignoredFindings: 0,
    baselineFindings: 0,
    groupedLineWarnings: 0,
    modeFilteredFindings: input.violations.length,
    maxDetailedFindings: 20,
    detailedFindingsTruncated: false,
    scanTruncated: input.scanTruncated ?? false,
    filesReviewed: input.filesReviewed,
    maxFiles: input.maxFiles,
    topScannedDirectories: [],
    findingHistogram: histogram,
    caveats: input.caveats ?? []
  };
  return {
    score: input.violations.length > 0 ? 82 : 100,
    grade: input.violations.length > 0 ? "B" : "A",
    mode: "audit",
    gate: gateFor(input.violations),
    summary: {
      totalFindings: input.violations.length,
      errors: input.violations.filter((violation) => violation.severity === "error").length,
      warnings: input.violations.filter((violation) => violation.severity === "warning").length,
      shown: input.violations.length,
      suppressed: 0,
      noiseSuppressed: 0,
      baselineSuppressed: 0,
      coverageCaveats: coverage.caveats
    },
    coverage,
    groups: groupsFor(histogram),
    priorityFindings: input.violations,
    violations: input.violations
  };
}

function groupsFor(histogram: ReviewFindingHistogramEntry[]): ReviewGroup[] {
  return histogram.map((entry) => ({
    key: entry.code,
    count: entry.count,
    severity: entry.severity,
    samplePaths: [],
    recommendation: "Use Foundry actionability before routing this finding."
  }));
}

function gateFor(violations: ReviewViolation[]): ReviewGate {
  return {
    status: violations.some((violation) => violation.severity === "error") ? "fail" : violations.length > 0 ? "warn" : "pass",
    reason: violations.length > 0 ? "Fixture review report contains findings." : "Fixture review report is clean.",
    thresholds: {
      maxErrors: 0,
      maxWarnings: 0,
      minScore: 90
    }
  };
}
