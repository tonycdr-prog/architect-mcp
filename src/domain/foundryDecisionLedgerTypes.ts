import type { FoundryActionabilityReport } from "./foundryActionabilityTypes.js";

export type FoundryDecisionRoute =
  | "pr_preview"
  | "architect_issue"
  | "exception"
  | "no_op"
  | "ask_human";

export type FoundryDecisionApprovalState =
  | "approval_required"
  | "human_required"
  | "not_required";

export type FoundryDecisionRedactionState =
  | "public"
  | "redacted"
  | "sensitive";

export type FoundryDecisionLedgerEntry = {
  id: string;
  route: FoundryDecisionRoute;
  evidenceIds: string[];
  source: {
    score: number;
  };
  decisionReason: string;
  redactionState: FoundryDecisionRedactionState;
  verificationRequirements: string[];
  approvalState: FoundryDecisionApprovalState;
  nextAction: string;
  mutation: {
    serverMutationAllowed: false;
    explicitApprovalRequired: boolean;
  };
};

export type FoundryDecisionLedgerReport = {
  schemaVersion: 1;
  ledgerId: string;
  recordedAt?: string;
  summary: {
    totalEntries: number;
    byRoute: Record<string, number>;
    approvalRequired: number;
    humanRequired: number;
    publicSafetyHolds: number;
    serverWritesPerformed: 0;
  };
  entries: FoundryDecisionLedgerEntry[];
  publicSafety: {
    rawPayloadsIncluded: false;
    rawRepoContentIncluded: false;
    localPathsIncluded: false;
    tokenValuesIncluded: false;
    mutationAllowed: false;
    publicLedgerOnly: true;
  };
};

export type FoundryDecisionLedgerInput = {
  actionability?: FoundryActionabilityReport | Record<string, unknown>;
  ledgerId?: string;
  recordedAt?: string;
};
