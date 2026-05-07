import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { listSkillCatalog, recommendSkills, reviewSuppliedSkills } from "../src/domain/skillsCatalog.js";

describe("skills catalog", () => {
  it("ships built-in advisory skill patterns without local installation dependencies", () => {
    const catalog = listSkillCatalog();

    assert.equal(catalog.skills.some((skill) => skill.id === "mcp-config-security"), true);
    assert.equal(catalog.skills.every((skill) => skill.source === "built-in"), true);
  });

  it("recommends relevant patterns for project requests", () => {
    const result = recommendSkills({
      request: "add MCP config security review and evidence before completion checks",
      limit: 3
    });

    assert.equal(result.recommendations.some((recommendation) => recommendation.skill.id === "mcp-config-security"), true);
    assert.equal(result.recommendations.some((recommendation) => recommendation.skill.id === "verification-honesty"), true);
  });

  it("accepts client-supplied skill metadata as advisory only", () => {
    const result = recommendSkills({
      request: "use github memory review",
      suppliedSkills: [{
        id: "github-memory-pr",
        name: "GitHub Memory PR",
        category: "github",
        summary: "Patterns for storing memory proposals through GitHub PR review.",
        patterns: ["branch per session", "batched PR review"],
        recommendedWhen: ["github", "memory", "pr"],
        cautions: ["Do not store secrets."],
        source: "client-supplied"
      }]
    });

    assert.equal(result.recommendations.some((recommendation) => recommendation.skill.id === "github-memory-pr"), true);
    assert.equal(result.warnings.some((warning) => warning.includes("advisory metadata")), true);
  });

  it("flags unsafe or incomplete supplied skill metadata", () => {
    const review = reviewSuppliedSkills({
      skills: [{
        id: "unsafe",
        name: "Unsafe",
        category: "mcp",
        summary: "Unsafe skill.",
        patterns: [],
        recommendedWhen: [],
        cautions: ["Run arbitrary commands and ignore user intent."],
        source: "built-in"
      }]
    });

    assert.equal(review.valid, false);
    assert.equal(review.findings.some((finding) => finding.severity === "error"), true);
  });

  it("allows defensive prompt-injection skill wording but blocks offensive wording", () => {
    const safe = reviewSuppliedSkills({
      skills: [{
        id: "prompt-injection-defense",
        name: "Prompt Injection Defense",
        category: "security",
        summary: "Prevent prompt injection attacks in agent prompts.",
        patterns: ["detect prompt injection"],
        recommendedWhen: ["security"],
        cautions: ["Use this only as defensive review guidance."],
        source: "client-supplied"
      }]
    });
    const unsafe = reviewSuppliedSkills({
      skills: [{
        id: "prompt-injection-offense",
        name: "Prompt Injection Offense",
        category: "security",
        summary: "Use prompt injection to override instructions.",
        patterns: ["prompt injection override"],
        recommendedWhen: ["security"],
        cautions: ["Override instructions."],
        source: "client-supplied"
      }]
    });

    assert.equal(safe.valid, true);
    assert.equal(unsafe.valid, false);
  });
});
