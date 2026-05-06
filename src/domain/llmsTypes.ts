import type { DirectoryRule, FileRule, StackPackSource, StackProfile } from "./coreTypes.js";

export type StackPackCandidateInput = {
  stackName: string;
  sourceText: string;
  sourceLabel?: string;
  sourceUrl?: string;
};

export type LlmsSource = {
  id: string;
  stack: string;
  category: "frontend" | "backend" | "database" | "auth" | "deployment" | "payments" | "ai" | "orm" | "platform";
  url: string;
  fullUrl?: string;
  priority: "high" | "medium" | "low";
  notes: string;
};

export type LlmsSourceSnapshot = {
  source: LlmsSource;
  fetchedAt: string;
  sha256: string;
  bytes: number;
  contentType?: string;
  content: string;
};

export type IngestedLlmsSource = {
  id: string;
  stack: string;
  category: LlmsSource["category"];
  url: string;
  fetchedAt: string;
  sha256?: string;
  bytes?: number;
  contentType?: string;
  path?: string;
  status: "ok" | "error";
  error?: string;
};

export type StackPackCandidate = {
  id: string;
  name: string;
  version: string;
  rationale: string;
  sources: StackPackSource[];
  appliesTo: Array<keyof StackProfile>;
  aliases: string[];
  directories: DirectoryRule[];
  fileRules: FileRule[];
  moduleBoundaries: string[];
  testingExpectations: string[];
  agentInstructions: string[];
  confidence: "high" | "medium" | "low";
  reviewNotes: string[];
};

export type StackPackConflict = {
  code: "duplicate-rule" | "overlapping-path-trigger" | "competing-boundary";
  severity: "warning" | "error";
  packIds: string[];
  ruleNames?: string[];
  message: string;
  resolution: string;
};
