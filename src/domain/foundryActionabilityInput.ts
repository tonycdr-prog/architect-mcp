import type { FoundryEvidenceInventory, FoundryEvidenceItem } from "./foundryEvidenceTypes.js";

export function isFoundryEvidenceInventory(value: unknown): value is FoundryEvidenceInventory {
  return isRecord(value) &&
    value.schemaVersion === 1 &&
    Array.isArray(value.evidence) &&
    value.evidence.every(isEvidenceRecord) &&
    isCoverageRecord(value.coverage);
}

function isEvidenceRecord(value: unknown): value is FoundryEvidenceItem {
  if (!isRecord(value) || !isRecord(value.sourceRef)) return false;
  return typeof value.id === "string" &&
    isKnownEvidenceKind(value.kind) &&
    isKnownSourceType(value.sourceType) &&
    typeof value.sourceRef.sourceId === "string" &&
    typeof value.sourceRef.sourceType === "string" &&
    isKnownConfidence(value.confidence) &&
    (value.severity === undefined || value.severity === "error" || value.severity === "warning" || value.severity === "info") &&
    (value.code === undefined || typeof value.code === "string") &&
    (value.path === undefined || typeof value.path === "string") &&
    typeof value.publicSummary === "string" &&
    (value.recommendation === undefined || typeof value.recommendation === "string") &&
    (value.publicSafetyClass === "public" || value.publicSafetyClass === "redacted" || value.publicSafetyClass === "sensitive") &&
    (value.redactionStatus === "none" || value.redactionStatus === "redacted" || value.redactionStatus === "omitted_raw_payload" || value.redactionStatus === "redacted_and_omitted_raw_payload") &&
    (value.suppressionCandidate === undefined || isSuppressionCandidate(value.suppressionCandidate));
}

function isCoverageRecord(value: unknown): value is FoundryEvidenceInventory["coverage"] {
  return isRecord(value) &&
    typeof value.scanTruncated === "boolean" &&
    typeof value.detailedFindingsTruncated === "boolean" &&
    (value.filesReviewed === undefined || typeof value.filesReviewed === "number") &&
    (value.maxFiles === undefined || typeof value.maxFiles === "number") &&
    Array.isArray(value.topScannedDirectories) &&
    value.topScannedDirectories.every((entry) => isRecord(entry) && typeof entry.directory === "string" && typeof entry.files === "number") &&
    Array.isArray(value.findingHistogram) &&
    value.findingHistogram.every((entry) => isRecord(entry) && typeof entry.code === "string" && typeof entry.severity === "string" && typeof entry.count === "number") &&
    Array.isArray(value.caveats) &&
    value.caveats.every((entry) => typeof entry === "string");
}

function isKnownEvidenceKind(value: unknown): value is FoundryEvidenceItem["kind"] {
  return value === "finding" ||
    value === "verification" ||
    value === "repo_signal" ||
    value === "coverage_caveat" ||
    value === "coverage_histogram";
}

function isKnownSourceType(value: unknown): value is FoundryEvidenceItem["sourceType"] {
  return value === "architect_review" ||
    value === "external_tool" ||
    value === "verification" ||
    value === "repo_constitution" ||
    value === "coverage";
}

function isKnownConfidence(value: unknown): value is FoundryEvidenceItem["confidence"] {
  return value === "high" || value === "medium" || value === "low";
}

function isSuppressionCandidate(value: unknown): boolean {
  return isRecord(value) &&
    isKnownSuppressionCategory(value.category) &&
    typeof value.reason === "string" &&
    typeof value.prerequisiteIssue === "string";
}

function isKnownSuppressionCategory(value: unknown): boolean {
  return value === "generated_file" ||
    value === "vendored_code" ||
    value === "fixture_or_test_data" ||
    value === "docs_example" ||
    value === "conventional_entrypoint" ||
    value === "repo_profile_mismatch";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
