import type { UntrustedInputSource, WorkGateBoundary, WorkGateBypassCase } from "./workGateThreatModel.js";

export type ThreatModelReferenceFinding = {
  caseId: string;
  field: "untrustedSources" | "affectedBoundaryIds";
  id: string;
  message: string;
};

export type ThreatModelReferenceReview = {
  valid: boolean;
  findings: ThreatModelReferenceFinding[];
};

export function validateThreatModelReferences(
  cases: readonly WorkGateBypassCase[],
  catalogs: {
    untrustedInputs: readonly UntrustedInputSource[];
    boundaries: readonly WorkGateBoundary[];
  }
): ThreatModelReferenceReview {
  const sourceIds = new Set(catalogs.untrustedInputs.map((source) => source.id));
  const boundaryIds = new Set(catalogs.boundaries.map((boundary) => boundary.id));
  const findings: ThreatModelReferenceFinding[] = [];

  for (const testCase of cases) {
    for (const sourceId of testCase.untrustedSources) {
      if (!sourceIds.has(sourceId)) {
        findings.push(referenceFinding(testCase.id, "untrustedSources", sourceId, "untrusted input source"));
      }
    }

    for (const boundaryId of testCase.affectedBoundaryIds) {
      if (!boundaryIds.has(boundaryId)) {
        findings.push(referenceFinding(testCase.id, "affectedBoundaryIds", boundaryId, "work-gate boundary"));
      }
    }
  }

  return {
    valid: findings.length === 0,
    findings
  };
}

function referenceFinding(
  caseId: string,
  field: ThreatModelReferenceFinding["field"],
  id: string,
  label: string
): ThreatModelReferenceFinding {
  return {
    caseId,
    field,
    id,
    message: `Bypass case ${caseId} references unknown ${label} ${id}.`
  };
}
