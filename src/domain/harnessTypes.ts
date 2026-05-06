import type { FileSummary, StackProfile } from "./coreTypes.js";
import type { ProposedFilePlan } from "./intakeTypes.js";
import type { LlmsSource } from "./llmsTypes.js";

export type HarnessMode = "strict" | "guided-yolo" | "full-yolo";
export type HarnessDecision = "proceed" | "proceed_with_assumptions" | "confirm_before_edit" | "block_until_clarified";
export type Stoplight = "green" | "yellow" | "red";
export type BlastRadius = "low" | "medium" | "high" | "critical";
export type ChangeType = "bug_fix" | "refactor" | "feature" | "styling" | "performance" | "security" | "data_schema" | "dependency_config" | "unknown";
export type HarnessVerificationStatus = "not_run" | "passed" | "failed" | "skipped";

export type HarnessEvidence = {
  kind: "error_output" | "file_context" | "repo_pattern" | "user_statement" | "stack_guidance";
  summary: string;
  source?: string;
};

export type AssumptionLedgerEntry = {
  statement: string;
  reason: string;
  confidence: "high" | "medium" | "low";
  risk: BlastRadius;
  invalidatedBy: string;
  affectedArea: string;
};

export type TriggeredStackGuidance = {
  sourceId: string;
  stack: string;
  category: LlmsSource["category"];
  status: "loaded" | "metadata-only" | "failed";
  summary: string;
  evidence: string[];
  ruleNames: string[];
  warning?: string;
};

export type HarnessIntentInput = {
  request: string;
  mode?: HarnessMode;
  stack?: StackProfile;
  selectedSourceIds?: string[];
  files?: FileSummary[];
  currentError?: string;
  proposedPlan?: ProposedFilePlan;
  verification?: string[];
  assumptionCount?: number;
};

export type HarnessIntentResult = {
  mode: HarnessMode;
  decision: HarnessDecision;
  stoplight: Stoplight;
  blastRadius: BlastRadius;
  changeType: ChangeType;
  confidence: "high" | "medium" | "low";
  interpretedProblem: string;
  intendedFix: string;
  plan: string[];
  assumptions: AssumptionLedgerEntry[];
  nonGoals: string[];
  verification: string[];
  evidence: HarnessEvidence[];
  triggeredGuidance: TriggeredStackGuidance[];
  escalationTerms: string[];
  plainLanguageOptions: string[];
  confirmationPrompt?: string;
  warnings: string[];
  handoffSummary: string;
};

export type PreEditContract = {
  id: string;
  createdAt: string;
  mode: HarnessMode;
  decision: HarnessDecision;
  interpretedProblem: string;
  intendedBehavior: string;
  changeType: ChangeType;
  blastRadius: BlastRadius;
  likelyFiles: string[];
  nonGoals: string[];
  assumptions: AssumptionLedgerEntry[];
  evidence: HarnessEvidence[];
  triggeredGuidance: TriggeredStackGuidance[];
  verificationChecks: string[];
  rollbackPlan: string;
  outputContract: string[];
};
