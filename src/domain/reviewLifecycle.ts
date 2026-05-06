import type { BaselineFinding, ReviewBaseline, ReviewLifecycle, ReviewViolation } from "./types.js";

export function classifyReviewLifecycle(findings: ReviewViolation[], baseline: ReviewBaseline = { findings: [] }): ReviewLifecycle {
  const baselineFindings: ReviewViolation[] = [];
  const acceptedFindings: ReviewViolation[] = [];
  const newFindings: ReviewViolation[] = [];
  const matchedBaseline = new Set<BaselineFinding>();

  for (const finding of findings) {
    const baselineFinding = baseline.findings.find((candidate) => matchesBaselineFinding(finding, candidate));
    if (!baselineFinding) {
      newFindings.push(finding);
      continue;
    }

    matchedBaseline.add(baselineFinding);
    if (baselineFinding.status === "accepted") {
      acceptedFindings.push(finding);
    } else {
      baselineFindings.push(finding);
    }
  }

  return {
    newFindings,
    baselineFindings,
    acceptedFindings,
    resolvedFindings: baseline.findings.filter((finding) => !matchedBaseline.has(finding))
  };
}

function matchesBaselineFinding(finding: ReviewViolation, baselineFinding: BaselineFinding): boolean {
  if (baselineFinding.code !== finding.code) return false;
  if ((finding.severity === "error" || finding.confidence === "high") && !baselineFinding.path && !baselineFinding.message) return false;
  if (baselineFinding.path && baselineFinding.path !== finding.path) return false;
  if (baselineFinding.message && baselineFinding.message !== finding.message) return false;
  return true;
}
