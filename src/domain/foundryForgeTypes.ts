import type { FoundryDecisionLedgerReport, FoundryDecisionRoute } from "./foundryDecisionLedgerTypes.js";
import type { RepoConstitution } from "./repoConstitutionTypes.js";

export type FoundryForgePreviewKind =
  | "pull_request"
  | "architect_issue"
  | "exception_record"
  | "no_op_record"
  | "human_question";

export type FoundryForgeSection = {
  heading: string;
  content: string;
};

export type FoundryForgePreview = {
  id: string;
  kind: FoundryForgePreviewKind;
  route: FoundryDecisionRoute;
  sourceDecisionId: string;
  evidenceIds: string[];
  title: string;
  body: string;
  sections: FoundryForgeSection[];
  verificationPlan: string[];
  releaseNoteImpact: string;
  maintainerFitRationale: string;
  disclaimer: string;
  warnings: string[];
  mutation: {
    serverMutationAllowed: false;
    explicitApprovalRequired: boolean;
  };
};

export type FoundryForgePreviewReport = {
  schemaVersion: 1;
  summary: {
    totalDecisions: number;
    previewsGenerated: number;
    pullRequestPreviews: number;
    architectIssuePreviews: number;
    exceptionRecords: number;
    noOpRecords: number;
    humanQuestions: number;
    skipped: number;
    serverWritesPerformed: 0;
  };
  previews: FoundryForgePreview[];
  publicSafety: {
    rawPayloadsIncluded: false;
    rawRepoContentIncluded: false;
    localPathsIncluded: false;
    tokenValuesIncluded: false;
    mutationAllowed: false;
    publicPreviewsOnly: true;
  };
};

export type FoundryForgePreviewInput = {
  ledger?: FoundryDecisionLedgerReport | Record<string, unknown>;
  repoConstitution?: RepoConstitution | Record<string, unknown>;
};
