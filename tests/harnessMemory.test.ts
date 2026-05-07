import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { interpretImplementationIntent, createPreEditContract } from "../src/domain/harness.js";
import { applyHarnessMemory, extractHarnessMemory, reviewMemoryRelevance } from "../src/domain/harnessMemory.js";

describe("stateless harness memory", () => {
  it("extracts GitHub-ready proposals without persisting anything", () => {
    const result = extractHarnessMemory({
      request: "I prefer guided-yolo, and maybe later we use a GitHub repo with PRs for memory.",
      projectName: "architect-mcp"
    });

    assert.equal(result.futureAdapter.storage, "none");
    assert.equal(result.futureAdapter.githubReady, true);
    assert.equal(result.proposals.some((proposal) => proposal.kind === "preference" && proposal.policyAction === "auto_store"), true);
    assert.equal(result.proposals.some((proposal) => proposal.kind === "decision" && proposal.policyAction === "batch_review"), true);
    assert.equal(result.proposals.every((proposal) => proposal.targetPath.length > 0), true);
  });

  it("turns harness assumptions and contracts into scoped memory proposals", () => {
    const intent = interpretImplementationIntent({
      request: "fix this with best practices"
    });
    const contract = createPreEditContract({
      intent,
      likelyFiles: ["src/domain/harness.ts"]
    });
    const result = extractHarnessMemory({
      intent,
      contract,
      projectName: "architect-mcp"
    });

    assert.equal(result.proposals.some((proposal) => proposal.kind === "assumption" && proposal.scope === "session"), true);
    assert.equal(result.proposals.some((proposal) => proposal.kind === "decision" && proposal.scope === "project"), true);
  });

  it("does not apply red or irrelevant memories and respects token budgets", () => {
    const extracted = extractHarnessMemory({
      request: "I prefer guided-yolo, and use GitHub PRs later for memory.",
      projectName: "architect-mcp"
    });
    const secret = extractHarnessMemory({
      request: "remember this API token secret-token-value",
      projectName: "architect-mcp"
    });
    const applied = applyHarnessMemory({
      request: "use the guided-yolo harness on this memory work",
      memories: [...extracted.proposals, ...secret.proposals],
      tokenBudget: 80
    });

    assert.equal(applied.selected.some((proposal) => proposal.sensitivity === "secret"), false);
    assert.equal(applied.selected.length > 0, true);
    assert.equal(applied.tokenEstimate <= 80, true);
    assert.equal(applied.disclosures.every((disclosure) => disclosure.startsWith("Using remembered")), true);
  });

  it("flags unsafe red memory policies before application", () => {
    const extracted = extractHarnessMemory({
      request: "remember this API token secret-token-value",
      projectName: "architect-mcp"
    });
    const review = reviewMemoryRelevance({
      request: "apply memory to this harness task",
      memories: extracted.proposals
    });

    assert.equal(review.valid, false);
    assert.equal(review.findings.some((finding) => finding.message.includes("secret-like")), true);
  });

  it("does not apply secret-shaped memories with inconsistent metadata", () => {
    const memory = {
      id: "unsafe",
      kind: "preference" as const,
      scope: "user" as const,
      statement: "Use API token sk-test-123 when calling services.",
      rationale: "Caller supplied unsafe memory metadata.",
      confidence: "high" as const,
      risk: "green" as const,
      sensitivity: "internal" as const,
      policyAction: "auto_store" as const,
      tags: ["agent-harness"],
      tokenEstimate: 20,
      source: { kind: "manual" as const, summary: "manual" },
      invalidatedBy: "Never",
      targetPath: "user/preference.jsonl"
    };
    const applied = applyHarnessMemory({
      request: "use agent harness memory",
      memories: [memory]
    });
    const review = reviewMemoryRelevance({
      request: "use agent harness memory",
      memories: [memory]
    });

    assert.equal(applied.selected.length, 0);
    assert.equal(applied.discarded[0]?.reason.includes("secret-like"), true);
    assert.equal(review.valid, false);
  });
});
