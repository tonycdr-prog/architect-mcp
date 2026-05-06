import type { ReviewBaseline, ReviewReport, ReviewViolation } from "./types.js";

export function createBaselineFromReport(report: ReviewReport): ReviewBaseline {
  return {
    findings: report.violations.map(toBaselineFinding)
  };
}

export function createBaselineFromFindings(findings: ReviewViolation[]): ReviewBaseline {
  return {
    findings: findings.map(toBaselineFinding)
  };
}

function toBaselineFinding(finding: ReviewViolation) {
  return {
    code: finding.code,
    path: finding.path,
    message: finding.message
  };
}
