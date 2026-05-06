import { createHash } from "node:crypto";
import type { HarnessIntentResult, MemoryApplicationResult, MemoryExtractionInput, MemoryKind, MemoryPolicy, MemoryPolicyAction, MemoryProposal, MemoryRelevanceInput, MemoryRisk, MemoryScope, MemorySensitivity, MemorySource, PreEditContract } from "./types.js";

const DEFAULT_POLICY: Required<MemoryPolicy> = {
  autoStoreGreen: true,
  batchYellow: true,
  confirmRed: true,
  maxRetrievedTokens: 1_200
};

export function extractHarnessMemory(input: MemoryExtractionInput): { proposals: MemoryProposal[]; warnings: string[]; futureAdapter: { storage: "none"; githubReady: boolean; note: string } } {
  const policy = { ...DEFAULT_POLICY, ...input.policy };
  const proposals = [
    ...fromRequest(input.request, input.projectName, policy),
    ...fromIntent(input.intent, input.projectName, policy),
    ...fromContract(input.contract, input.projectName, policy),
    ...fromSessionSummary(input.sessionSummary, input.projectName, policy)
  ].filter((proposal, index, all) => all.findIndex((candidate) => candidate.statement === proposal.statement && candidate.scope === proposal.scope) === index);

  return {
    proposals,
    warnings: proposals.some((proposal) => proposal.sensitivity === "secret")
      ? ["Secret-looking memory proposals were marked discard; do not persist them."]
      : [],
    futureAdapter: {
      storage: "none",
      githubReady: true,
      note: "V1 is stateless. These proposals can later be written to a user-owned GitHub memory repo through a branch/PR adapter."
    }
  };
}

export function applyHarnessMemory(input: MemoryRelevanceInput): MemoryApplicationResult {
  const tokenBudget = input.tokenBudget ?? DEFAULT_POLICY.maxRetrievedTokens;
  const ranked = input.memories
    .map((memory) => ({ memory, score: relevanceScore(input.request, input.stack, memory) }))
    .sort((a, b) => b.score - a.score);
  const selected: MemoryProposal[] = [];
  const discarded: Array<{ id: string; reason: string }> = [];
  let tokenEstimate = 0;

  for (const item of ranked) {
    if (item.memory.policyAction === "discard") {
      discarded.push({ id: item.memory.id, reason: "Memory policy says discard." });
    } else if (item.memory.risk === "red") {
      discarded.push({ id: item.memory.id, reason: "Red memory needs explicit confirmation before use." });
    } else if (item.score <= 0) {
      discarded.push({ id: item.memory.id, reason: "Not relevant to the current request." });
    } else if (tokenEstimate + item.memory.tokenEstimate > tokenBudget) {
      discarded.push({ id: item.memory.id, reason: "Skipped to stay within the memory token budget." });
    } else {
      selected.push(item.memory);
      tokenEstimate += item.memory.tokenEstimate;
    }
  }

  return {
    selected,
    discarded,
    disclosures: selected.map((memory) => `Using remembered ${memory.scope} ${memory.kind}: ${memory.statement}`),
    tokenEstimate,
    warnings: selected.some((memory) => memory.risk === "yellow")
      ? ["Yellow memory is advisory; current user instructions still win."]
      : []
  };
}

export function reviewMemoryRelevance(input: MemoryRelevanceInput): { valid: boolean; findings: Array<{ id: string; severity: "warning" | "error"; message: string; recommendation: string }> } {
  const findings = input.memories.flatMap((memory) => {
    const memoryFindings: Array<{ id: string; severity: "warning" | "error"; message: string; recommendation: string }> = [];
    const score = relevanceScore(input.request, input.stack, memory);
    if (memory.sensitivity === "secret") {
      memoryFindings.push({
        id: memory.id,
        severity: "error",
        message: "Memory appears secret-like and should not be persisted or applied.",
        recommendation: "Discard this memory and store only non-sensitive policy or preference summaries."
      });
    }
    if (memory.risk === "red" && memory.policyAction !== "confirm_now" && memory.policyAction !== "discard") {
      memoryFindings.push({
        id: memory.id,
        severity: "error",
        message: "Red memory must not be auto-stored or silently applied.",
        recommendation: "Require explicit user confirmation or discard the memory."
      });
    }
    if (score <= 0) {
      memoryFindings.push({
        id: memory.id,
        severity: "warning",
        message: "Memory does not appear relevant to the current request.",
        recommendation: "Do not load this memory into the agent context for this turn."
      });
    }
    return memoryFindings;
  });

  return {
    valid: findings.every((finding) => finding.severity !== "error"),
    findings
  };
}

function fromRequest(request: string | undefined, projectName: string | undefined, policy: Required<MemoryPolicy>): MemoryProposal[] {
  if (!request?.trim()) return [];
  const normalized = request.trim();
  const proposals: MemoryProposal[] = [];
  if (/guided-yolo|yolo mode|confirm|review/i.test(normalized)) {
    proposals.push(createProposal({
      kind: "preference",
      scope: "user",
      statement: summarizePreference(normalized),
      rationale: "The user described how they want agents to balance speed and confirmation.",
      confidence: "medium",
      risk: "green",
      sensitivity: classifySensitivity(normalized),
      tags: ["agent-harness", "confirmation-policy"],
      source: { kind: "user_statement", summary: normalized },
      invalidatedBy: "The user later asks for a different confirmation or autonomy policy.",
      projectName,
      policy
    }));
  }
  if (/github|repo|pull request|pr|memory|remember/i.test(normalized)) {
    proposals.push(createProposal({
      kind: "decision",
      scope: "project",
      statement: "Memory persistence should be designed as an optional adapter, with user-owned GitHub repo support deferred behind explicit consent.",
      rationale: "The user wants memory without slowing the harness or forcing hosted storage.",
      confidence: "medium",
      risk: "yellow",
      sensitivity: classifySensitivity(normalized),
      tags: ["memory", "github-adapter", "local-first"],
      source: { kind: "user_statement", summary: normalized },
      invalidatedBy: "The project chooses a database-first or hosted-only memory architecture.",
      projectName,
      policy
    }));
  }
  return proposals;
}

function fromIntent(intent: HarnessIntentResult | undefined, projectName: string | undefined, policy: Required<MemoryPolicy>): MemoryProposal[] {
  if (!intent) return [];
  return intent.assumptions.map((assumption) => createProposal({
    kind: "assumption",
    scope: "session",
    statement: assumption.statement,
    rationale: assumption.reason,
    confidence: assumption.confidence,
    risk: assumption.risk === "low" ? "green" : assumption.risk === "medium" ? "yellow" : "red",
    sensitivity: "internal",
    tags: ["assumption", intent.changeType, intent.blastRadius],
    source: { kind: "harness_intent", summary: intent.handoffSummary },
    invalidatedBy: assumption.invalidatedBy,
    projectName,
    policy
  }));
}

function fromContract(contract: PreEditContract | undefined, projectName: string | undefined, policy: Required<MemoryPolicy>): MemoryProposal[] {
  if (!contract) return [];
  return [
    createProposal({
      kind: "decision",
      scope: "project",
      statement: `For ${contract.changeType} work, keep edits within ${contract.likelyFiles.slice(0, 4).join(", ")} unless scope is reconfirmed.`,
      rationale: "The pre-edit contract captured the intended blast radius for this implementation.",
      confidence: "high",
      risk: contract.blastRadius === "low" ? "green" : "yellow",
      sensitivity: "internal",
      tags: ["pre-edit-contract", contract.changeType, contract.blastRadius],
      source: { kind: "pre_edit_contract", summary: contract.interpretedProblem, reference: contract.id },
      invalidatedBy: "The user approves a wider contract or architecture direction.",
      projectName,
      policy
    })
  ];
}

function fromSessionSummary(sessionSummary: string | undefined, projectName: string | undefined, policy: Required<MemoryPolicy>): MemoryProposal[] {
  if (!sessionSummary?.trim()) return [];
  const summary = sessionSummary.trim();
  return [createProposal({
    kind: "session_summary",
    scope: "session",
    statement: summary.slice(0, 320),
    rationale: "Session summaries can help the next agent continue without loading the full transcript.",
    confidence: "medium",
    risk: "green",
    sensitivity: classifySensitivity(summary),
    tags: ["handoff", "session-summary"],
    source: { kind: "session_summary", summary: summary.slice(0, 240) },
    invalidatedBy: "The next session summary supersedes this handoff.",
    projectName,
    policy
  })];
}

function createProposal(input: {
  kind: MemoryKind;
  scope: MemoryScope;
  statement: string;
  rationale: string;
  confidence: "high" | "medium" | "low";
  risk: MemoryRisk;
  sensitivity: MemorySensitivity;
  tags: string[];
  source: MemorySource;
  invalidatedBy: string;
  projectName: string | undefined;
  policy: Required<MemoryPolicy>;
}): MemoryProposal {
  const sensitivity = input.sensitivity;
  const risk = sensitivity === "secret" ? "red" : input.risk;
  const targetPath = pathFor(input.scope, input.kind, input.projectName);
  const statement = input.statement.trim();
  return {
    id: createHash("sha256").update(`${input.scope}:${input.kind}:${statement}`).digest("hex").slice(0, 12),
    kind: input.kind,
    scope: input.scope,
    statement,
    rationale: input.rationale,
    confidence: input.confidence,
    risk,
    sensitivity,
    policyAction: policyActionFor(risk, sensitivity, input.policy),
    tags: input.tags,
    tokenEstimate: estimateTokens(`${statement} ${input.rationale} ${input.tags.join(" ")}`),
    source: input.source,
    invalidatedBy: input.invalidatedBy,
    targetPath,
    reviewNote: risk === "yellow" ? "Batch into an async memory review." : risk === "red" ? "Ask before storing or applying." : undefined
  };
}

function policyActionFor(risk: MemoryRisk, sensitivity: MemorySensitivity, policy: Required<MemoryPolicy>): MemoryPolicyAction {
  if (sensitivity === "secret") return "discard";
  if (risk === "green") return policy.autoStoreGreen ? "auto_store" : "batch_review";
  if (risk === "yellow") return policy.batchYellow ? "batch_review" : "confirm_now";
  return policy.confirmRed ? "confirm_now" : "discard";
}

function pathFor(scope: MemoryScope, kind: MemoryKind, projectName = "default-project"): string {
  const project = slug(projectName);
  if (scope === "user") return `user/${kind}.jsonl`;
  if (scope === "project") return `projects/${project}/${kind}.jsonl`;
  if (scope === "codebase") return `projects/${project}/codebase/${kind}.jsonl`;
  return `sessions/${kind}.jsonl`;
}

function relevanceScore(request: string, stack: MemoryRelevanceInput["stack"], memory: MemoryProposal): number {
  const requestTerms = terms(`${request} ${Object.values(stack ?? {}).join(" ")}`);
  const memoryTerms = terms(`${memory.statement} ${memory.rationale} ${memory.tags.join(" ")}`);
  let score = memory.tags.filter((tag) => requestTerms.has(tag)).length * 3;
  for (const term of requestTerms) {
    if (memoryTerms.has(term)) score += 1;
  }
  if (memory.scope === "user") score += 1;
  if (memory.scope === "project") score += 2;
  return score;
}

function terms(value: string): Set<string> {
  return new Set(value.toLowerCase().match(/[a-z0-9][a-z0-9_-]{2,}/g) ?? []);
}

function classifySensitivity(value: string): MemorySensitivity {
  if (/api[_-]?key|secret|token|password|private key|-----begin/i.test(value)) return "secret";
  if (/customer|tenant|payment|auth|security|permission/i.test(value)) return "sensitive";
  return "internal";
}

function summarizePreference(value: string): string {
  if (/guided-yolo|yolo/i.test(value)) return "User prefers agents to move fast in guided-yolo mode while pausing for vague risky edits.";
  if (/confirm|review/i.test(value)) return "User wants confirmation for risky or ambiguous long-lived agent behavior.";
  return value.slice(0, 240);
}

function estimateTokens(value: string): number {
  return Math.max(12, Math.ceil(value.length / 4));
}

function slug(value: string | undefined): string {
  return (value ?? "default-project").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "default-project";
}
