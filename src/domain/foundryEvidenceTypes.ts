import type { RepoConstitution } from "./repoConstitutionTypes.js";
import type { ReviewReport, ReviewViolation } from "./types.js";

export type FoundryEvidenceSourceType =
  | "architect_review"
  | "external_tool"
  | "verification"
  | "repo_constitution"
  | "coverage";

export type FoundryEvidenceKind =
  | "finding"
  | "verification"
  | "repo_signal"
  | "coverage_caveat"
  | "coverage_histogram";

export type FoundryPublicSafetyClass = "public" | "redacted" | "sensitive";

export type FoundryRedactionStatus =
  | "none"
  | "redacted"
  | "omitted_raw_payload"
  | "redacted_and_omitted_raw_payload";

export type FoundrySuppressionCategory =
  | "generated_file"
  | "vendored_code"
  | "fixture_or_test_data"
  | "docs_example"
  | "conventional_entrypoint"
  | "repo_profile_mismatch";

export type FoundryExternalFinding = {
  toolName: string;
  ruleId?: string;
  severity?: "error" | "warning" | "info";
  confidence?: "high" | "medium" | "low";
  path?: string;
  message: string;
  recommendation?: string;
  rawPayload?: unknown;
  securitySensitive?: boolean;
};

export type FoundryVerificationEvidence = {
  check: string;
  status: "passed" | "failed" | "skipped" | "not_run" | "unknown";
  summary?: string;
  recordedAt?: string;
};

export type FoundryEvidenceInventoryInput = {
  findings?: ReviewViolation[];
  reviewReports?: ReviewReport[];
  externalFindings?: FoundryExternalFinding[];
  verification?: FoundryVerificationEvidence[];
  repoConstitution?: RepoConstitution;
};

export type FoundryEvidenceSourceRef = {
  sourceType: FoundryEvidenceSourceType;
  sourceId: string;
  index?: number;
  path?: string;
};

export type FoundrySuppressionCandidate = {
  category: FoundrySuppressionCategory;
  reason: string;
  prerequisiteIssue: string;
};

export type FoundryEvidenceItem = {
  id: string;
  kind: FoundryEvidenceKind;
  sourceType: FoundryEvidenceSourceType;
  sourceRef: FoundryEvidenceSourceRef;
  confidence: "high" | "medium" | "low";
  severity?: "error" | "warning" | "info";
  code?: string;
  path?: string;
  publicSummary: string;
  recommendation?: string;
  publicSafetyClass: FoundryPublicSafetyClass;
  redactionStatus: FoundryRedactionStatus;
  suppressionCandidate?: FoundrySuppressionCandidate;
};

export type FoundrySuppressionPrerequisite = {
  category: FoundrySuppressionCategory;
  issue: string;
  reason: string;
};

export type FoundryEvidenceInventory = {
  schemaVersion: 1;
  summary: {
    totalEvidence: number;
    bySourceType: Record<string, number>;
    byConfidence: Record<string, number>;
    byPublicSafetyClass: Record<string, number>;
    redacted: number;
    omittedRawPayloads: number;
    suppressionCandidates: number;
    coverageCaveats: number;
  };
  evidence: FoundryEvidenceItem[];
  coverage: {
    scanTruncated: boolean;
    detailedFindingsTruncated: boolean;
    filesReviewed?: number;
    maxFiles?: number;
    topScannedDirectories: Array<{ directory: string; files: number }>;
    findingHistogram: Array<{ code: string; severity: string; count: number }>;
    caveats: string[];
  };
  suppressionPrerequisites: FoundrySuppressionPrerequisite[];
  publicSafety: {
    rawPayloadsIncluded: false;
    rawRepoContentIncluded: false;
    mutationAllowed: false;
  };
};
