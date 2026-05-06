import type { StackProfile } from "./coreTypes.js";
import type { HarnessIntentResult, PreEditContract } from "./harnessTypes.js";

export type MemoryScope = "user" | "project" | "session" | "codebase";
export type MemoryKind = "preference" | "decision" | "assumption" | "stack_guidance" | "anti_pattern" | "session_summary" | "repo_fact" | "accepted_risk";
export type MemoryRisk = "green" | "yellow" | "red";
export type MemoryPolicyAction = "auto_store" | "batch_review" | "confirm_now" | "discard";
export type MemorySensitivity = "public" | "internal" | "sensitive" | "secret";

export type MemorySource = {
  kind: "harness_intent" | "pre_edit_contract" | "implementation_review" | "user_statement" | "session_summary" | "manual";
  summary: string;
  reference?: string;
};

export type MemoryProposal = {
  id: string;
  kind: MemoryKind;
  scope: MemoryScope;
  statement: string;
  rationale: string;
  confidence: "high" | "medium" | "low";
  risk: MemoryRisk;
  sensitivity: MemorySensitivity;
  policyAction: MemoryPolicyAction;
  tags: string[];
  tokenEstimate: number;
  source: MemorySource;
  invalidatedBy: string;
  targetPath: string;
  reviewNote?: string;
  expiresAt?: string;
};

export type MemoryPolicy = {
  autoStoreGreen?: boolean;
  batchYellow?: boolean;
  confirmRed?: boolean;
  maxRetrievedTokens?: number;
};

export type MemoryExtractionInput = {
  request?: string;
  intent?: HarnessIntentResult;
  contract?: PreEditContract;
  sessionSummary?: string;
  projectName?: string;
  policy?: MemoryPolicy;
};

export type MemoryRelevanceInput = {
  request: string;
  stack?: StackProfile;
  memories: MemoryProposal[];
  tokenBudget?: number;
};

export type MemoryApplicationResult = {
  selected: MemoryProposal[];
  discarded: Array<{ id: string; reason: string }>;
  disclosures: string[];
  tokenEstimate: number;
  warnings: string[];
};
