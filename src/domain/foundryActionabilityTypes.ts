import type { FoundryEvidenceInventory, FoundryEvidenceItem } from "./foundryEvidenceTypes.js";

export type FoundryActionabilityDecision =
  | "pr_preview_candidate"
  | "ask_human"
  | "exception_candidate"
  | "no_op_candidate";

export type FoundryActionabilityFactorName =
  | "evidence_strength"
  | "confidence"
  | "blast_radius"
  | "patch_size"
  | "maintainer_fit"
  | "duplicate_risk"
  | "release_impact"
  | "verification_path"
  | "public_safety_risk"
  | "maintainer_value";

export type FoundryActionabilityFactor = {
  name: FoundryActionabilityFactorName;
  score: number;
  status: "strong" | "mixed" | "weak" | "blocked";
  rationale: string;
};

export type FoundryActionabilityAssessment = {
  evidenceId: string;
  sourceType: FoundryEvidenceItem["sourceType"];
  code?: string;
  path?: string;
  decision: FoundryActionabilityDecision;
  score: number;
  publicRationale: string[];
  requiredVerification: string[];
  blockers: string[];
  factors: FoundryActionabilityFactor[];
};

export type FoundryActionabilityReport = {
  schemaVersion: 1;
  summary: {
    totalFindings: number;
    byDecision: Record<string, number>;
    prPreviewCandidates: number;
    askHuman: number;
    exceptionCandidates: number;
    noOpCandidates: number;
    publicSafetyHolds: number;
  };
  assessments: FoundryActionabilityAssessment[];
  publicSafety: {
    rawPayloadsIncluded: false;
    rawRepoContentIncluded: false;
    localPathsIncluded: false;
    tokenValuesIncluded: false;
    mutationAllowed: false;
    publicRecommendationsOnly: true;
  };
};

export type FoundryActionabilityInput = {
  inventory?: FoundryEvidenceInventory | Record<string, unknown>;
  verificationHints?: string[];
};
