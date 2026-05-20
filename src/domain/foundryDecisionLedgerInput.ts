import type { FoundryActionabilityAssessment, FoundryActionabilityReport } from "./foundryActionabilityTypes.js";

export function isFoundryActionabilityReport(value: unknown): value is FoundryActionabilityReport {
  return isRecord(value) &&
    value.schemaVersion === 1 &&
    Array.isArray(value.assessments) &&
    value.assessments.every(isAssessmentRecord);
}

function isAssessmentRecord(value: unknown): value is FoundryActionabilityAssessment {
  if (!isRecord(value)) return false;
  return typeof value.evidenceId === "string" &&
    isKnownDecision(value.decision) &&
    typeof value.sourceType === "string" &&
    typeof value.score === "number" &&
    (value.code === undefined || typeof value.code === "string") &&
    (value.path === undefined || typeof value.path === "string") &&
    Array.isArray(value.publicRationale) &&
    value.publicRationale.every((item) => typeof item === "string") &&
    Array.isArray(value.requiredVerification) &&
    value.requiredVerification.every((item) => typeof item === "string") &&
    Array.isArray(value.blockers) &&
    value.blockers.every((item) => typeof item === "string") &&
    Array.isArray(value.factors) &&
    value.factors.every(isFactorRecord);
}

function isFactorRecord(value: unknown): boolean {
  return isRecord(value) &&
    typeof value.name === "string" &&
    typeof value.score === "number" &&
    typeof value.status === "string" &&
    typeof value.rationale === "string";
}

function isKnownDecision(value: unknown): boolean {
  return value === "pr_preview_candidate" ||
    value === "ask_human" ||
    value === "exception_candidate" ||
    value === "no_op_candidate";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
