import { scoreAgentInstructions, scoreLlmsTxt } from "./artifactQuality.js";
import type { ReviewViolation } from "./types.js";

export type OutputMode = "compact" | "standard" | "full";

export function selectLocalOrchestrationRecipe(input: { request?: string; risk?: string } = {}) {
  const text = `${input.request ?? ""} ${input.risk ?? ""}`.toLowerCase();
  const id = /stack.*pack|llms/.test(text) ? "stack-pack-promotion"
    : /docs|readme|llms|agents/.test(text) ? "documentation-refresh"
      : /security|auth|secret|payment/.test(text) ? "security-sensitive-change"
      : /refactor/.test(text) ? "risky-refactor"
        : /bug|fix/.test(text) ? "vague-bug-fix"
          : "existing-repo-review";
  return {
    recipe: {
      id,
      requiredInputs: ["request", "file summaries or brief when available"],
      tools: recipeTools(id),
      gates: ["intent clarity", "pre-edit contract when risky", "review gate", "verification honesty"],
      expectedOutputs: ["normalized result", "handoff", "proof"],
      proofRequirements: ["named checks", "review result", "not-done disclosure"]
    },
    warnings: /hosted|github|database persistence|billing|account|dashboard/i.test(text) ? ["Request mentions deferred productization; keep this recipe local-only."] : []
  };
}

export function evaluateScenarioAcceptance(input: { scenario?: string; intentReady?: boolean; contractReady?: boolean; reviewPassed?: boolean; verified?: boolean; finalResponseHonest?: boolean; artifactScores?: Array<{ status?: string }> } = {}) {
  const checks = [
    { id: "intent", passed: input.intentReady !== false },
    { id: "contract", passed: input.contractReady !== false },
    { id: "review", passed: input.reviewPassed !== false },
    { id: "verification", passed: input.verified === true },
    { id: "final-response", passed: input.finalResponseHonest !== false },
    { id: "artifacts", passed: (input.artifactScores ?? []).every((score) => score.status !== "fail") }
  ];
  const failed = checks.filter((check) => !check.passed);
  return {
    scenario: input.scenario ?? "general-local-run",
    status: failed.length ? "fail" : "pass",
    stoplight: failed.length ? "red" : "green",
    checks,
    plainEnglish: failed.length ? `Scenario is not accepted yet: ${failed.map((check) => check.id).join(", ")} needs work.` : "Scenario meets the local operating-model acceptance profile."
  };
}

export function normalizeMcpResult(input: { status?: string; findings?: ReviewViolation[]; evidence?: string[]; assumptions?: string[]; warnings?: string[]; notDone?: string[]; handoff?: string } = {}) {
  const findings = input.findings ?? [];
  return {
    status: input.status ?? (findings.some((finding) => finding.severity === "error") ? "fail" : findings.length ? "warn" : "pass"),
    stoplight: findings.some((finding) => finding.severity === "error") ? "red" : findings.length ? "yellow" : "green",
    findings,
    evidence: input.evidence ?? [],
    assumptions: input.assumptions ?? [],
    nextActions: findings.slice(0, 3).map((finding) => finding.recommendation),
    proof: input.evidence?.length ? input.evidence : ["Run or cite relevant verification before claiming completion."],
    warnings: input.warnings ?? [],
    notDone: input.notDone ?? [],
    handoff: input.handoff ?? "Continue with the selected local recipe and preserve evidence links."
  };
}

export function planContextBudget(input: { mode?: OutputMode; requestedTokens?: number; findings?: ReviewViolation[] } = {}) {
  const mode = input.mode ?? "standard";
  const budget = input.requestedTokens ?? (mode === "compact" ? 600 : mode === "full" ? 2400 : 1200);
  return {
    mode,
    budget,
    precedence: ["blockers", "assumptions", "proof", "next actions", "optional teaching"],
    evidenceLimit: mode === "compact" ? 3 : mode === "full" ? 12 : 6,
    warning: (input.findings?.length ?? 0) * 80 > budget ? "Finding detail may exceed budget; summarize by cluster." : undefined
  };
}

export function routeEvidence(input: { findings?: ReviewViolation[]; sources?: Array<{ id: string; snapshotPath?: string; sha256?: string; fetchedAt?: string }>; verification?: Array<{ check: string; status: string }> } = {}) {
  const sources = input.sources ?? [];
  return {
    evidence: (input.findings ?? []).map((finding, index) => ({
      id: `ev-${index + 1}`,
      findingCode: finding.code,
      path: finding.path,
      message: finding.message,
      source: sources[index % Math.max(1, sources.length)],
      verification: input.verification?.[index % Math.max(1, input.verification.length)]
    })),
    warnings: sources.some((source) => !source.snapshotPath || !source.sha256) ? ["Some source provenance is missing snapshot path or hash."] : []
  };
}

export function createLocalDryRunPlan(input: { request?: string; risky?: boolean; expectedArtifacts?: { agentsMd?: string; llmsTxt?: string } } = {}) {
  const recipe = selectLocalOrchestrationRecipe({ request: input.request });
  return {
    recipe: recipe.recipe,
    gates: input.risky ? ["confirm before edit", "pre-edit contract", "review", "verification"] : ["ready to edit", "review", "verification"],
    selectedStandards: ["foundation packs", "triggered stack packs", "advisory skill patterns"],
    verificationChecks: ["typecheck/test/build when available", "review_repo_structure", "review_agent_final_response"],
    artifactChecks: {
      agentsMd: input.expectedArtifacts?.agentsMd ? scoreAgentInstructions(input.expectedArtifacts.agentsMd).status : "not_supplied",
      llmsTxt: input.expectedArtifacts?.llmsTxt ? scoreLlmsTxt(input.expectedArtifacts.llmsTxt).status : "not_supplied"
    },
    finalResponseContract: ["changed", "verified", "assumptions", "not done"]
  };
}

export function reviewToolLoopQuality(input: { toolsRun?: string[]; risky?: boolean; finalResponse?: string; verification?: Array<{ status: string }> } = {}) {
  const tools = new Set(input.toolsRun ?? []);
  const findings: string[] = [];
  if (input.risky && !tools.has("interpret_implementation_intent")) findings.push("Risky work skipped intent interpretation.");
  if (input.risky && !tools.has("create_pre_edit_contract")) findings.push("Risky work skipped pre-edit contract.");
  if (!tools.has("review_repo_structure") && !tools.has("review_implementation_against_contract")) findings.push("Implementation was not reviewed.");
  if (!(input.verification ?? []).some((check) => check.status === "passed")) findings.push("No passed verification check supplied.");
  if (input.finalResponse) {
    const missingSections = ["changed", "verified", "assumptions", "not done"].filter((section) => !new RegExp(section, "i").test(input.finalResponse ?? ""));
    if (missingSections.length > 0) findings.push(`Final response omits output-contract sections: ${missingSections.join(", ")}.`);
  }
  return {
    status: findings.length ? "fail" : "pass",
    findings,
    correctiveNextStep: findings[0] ?? "Tool loop quality looks acceptable."
  };
}

function recipeTools(id: string): string[] {
  if (id === "stack-pack-promotion") return ["derive_stack_pack_from_llms_source", "review_stack_pack_candidate", "analyze_stack_pack_conflicts"];
  if (id === "documentation-refresh") return ["score_agent_artifacts", "review_documentation_intelligence"];
  if (id === "security-sensitive-change") return ["interpret_implementation_intent", "create_pre_edit_contract", "review_implementation_against_contract", "review_agent_final_response"];
  if (id === "risky-refactor") return ["interpret_implementation_intent", "create_pre_edit_contract", "review_implementation_against_contract"];
  return ["grill_me", "review_repo_structure", "review_agent_final_response"];
}
