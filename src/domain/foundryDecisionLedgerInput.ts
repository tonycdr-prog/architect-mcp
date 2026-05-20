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
    isKnownFactorName(value.name) &&
    typeof value.score === "number" &&
    Number.isFinite(value.score) &&
    isKnownFactorStatus(value.status) &&
    typeof value.rationale === "string";
}

function isKnownDecision(value: unknown): boolean {
  return value === "pr_preview_candidate" ||
    value === "ask_human" ||
    value === "exception_candidate" ||
    value === "no_op_candidate";
}

function isKnownFactorName(value: unknown): boolean {
  return value === "evidence_strength" ||
    value === "confidence" ||
    value === "blast_radius" ||
    value === "patch_size" ||
    value === "maintainer_fit" ||
    value === "duplicate_risk" ||
    value === "release_impact" ||
    value === "verification_path" ||
    value === "public_safety_risk" ||
    value === "maintainer_value";
}

function isKnownFactorStatus(value: unknown): boolean {
  return value === "strong" ||
    value === "mixed" ||
    value === "weak" ||
    value === "blocked";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
