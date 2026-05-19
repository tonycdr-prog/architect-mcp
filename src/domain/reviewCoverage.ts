import type { ReviewOptions, ReviewReport, ReviewViolation } from "./types.js";

type CoverageInput = {
  violations: ReviewViolation[];
  eligible: ReviewViolation[];
  violationsForMode: ReviewViolation[];
  ignored: ReviewViolation[];
  baselineFindings: ReviewViolation[];
  groupedLineWarnings: ReviewViolation[];
  suppressed: number;
  modeFilteredFindings: number;
  maxDetailedFindings: number;
  scan: ReviewOptions["scan"];
};

export function createReviewCoverage(input: CoverageInput): ReviewReport["coverage"] {
  const detailedFindingsTruncated = input.modeFilteredFindings > input.violationsForMode.length;
  const scanTruncated = input.scan?.truncated ?? false;
  const caveats = [
    ...(scanTruncated ? [scanTruncationCaveat(input.scan)] : []),
    ...(detailedFindingsTruncated ? [`Detailed findings were capped at ${input.maxDetailedFindings}; use report.coverage.findingHistogram and report.groups for the full distribution.`] : [])
  ];

  return {
    totalFindings: input.violations.length,
    eligibleFindings: input.eligible.length,
    detailedFindings: input.violationsForMode.length,
    suppressedFindings: input.suppressed,
    ignoredFindings: input.ignored.length,
    baselineFindings: input.baselineFindings.length,
    groupedLineWarnings: input.groupedLineWarnings.length,
    modeFilteredFindings: input.modeFilteredFindings,
    maxDetailedFindings: input.maxDetailedFindings,
    detailedFindingsTruncated,
    scanTruncated,
    filesReviewed: input.scan?.filesReviewed,
    maxFiles: input.scan?.maxFiles,
    topScannedDirectories: input.scan?.topScannedDirectories ?? [],
    findingHistogram: createFindingHistogram(input.violations),
    caveats
  };
}

function scanTruncationCaveat(scan: ReviewOptions["scan"]): string {
  const reviewed = scan?.filesReviewed;
  const maxFiles = scan?.maxFiles;
  if (reviewed && maxFiles) return `Workspace scan reached ${reviewed}/${maxFiles} files; findings describe only the scanned subset.`;
  if (maxFiles) return `Workspace scan reached the ${maxFiles}-file limit; findings describe only the scanned subset.`;
  return "Workspace scan was truncated; findings describe only the scanned subset.";
}

function createFindingHistogram(violations: ReviewViolation[]): ReviewReport["coverage"]["findingHistogram"] {
  const counts = new Map<string, ReviewReport["coverage"]["findingHistogram"][number]>();

  for (const violation of violations) {
    const key = `${violation.code}:${violation.severity}`;
    const existing = counts.get(key);
    if (existing) {
      existing.count += 1;
      continue;
    }
    counts.set(key, {
      code: violation.code,
      severity: violation.severity,
      count: 1
    });
  }

  return [...counts.values()].sort((left, right) =>
    right.count - left.count ||
    left.code.localeCompare(right.code) ||
    severitySort(left.severity) - severitySort(right.severity)
  );
}

function severitySort(severity: ReviewViolation["severity"]): number {
  return severity === "error" ? 0 : 1;
}
