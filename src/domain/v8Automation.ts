import { listFoundationPacks } from "./foundationPacks.js";
import { matchesPathPattern } from "./pathRules.js";
import { scoreAgentInstructions, scoreLlmsTxt } from "./artifactQuality.js";
import { reviewAgentFinalResponse } from "./finalResponseReview.js";
import { reviewMcpConfigSecurity } from "./mcpSecurity.js";
import { reviewFileSummaries } from "./reviewer.js";
import type { ProjectBrief, ReviewReport, ReviewViolation, StackPack } from "./types.js";

export function reviewStandardsRefactor(input: { stackPacks?: StackPack[] } = {}) {
  const packs = input.stackPacks ?? [];
  const rules = packs.flatMap((pack) => pack.fileRules.map((rule) => ({ packId: pack.id, rule })));
  const seen = new Map<string, string>();
  const suggestions: Array<{ kind: string; message: string; risk: string }> = [];
  for (const item of rules) {
    const key = item.rule.rule.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
    const previous = seen.get(key);
    if (previous && previous !== item.packId) suggestions.push({ kind: "merge", message: `${previous} and ${item.packId} contain overlapping rules.`, risk: "medium" });
    seen.set(key, item.packId);
    if (!item.rule.goodExample || !item.rule.badExample) suggestions.push({ kind: "strengthen-examples", message: `${item.packId}/${item.rule.name} needs examples.`, risk: "low" });
    if (item.rule.triggerKind === "manual-review") suggestions.push({ kind: "detector-family", message: `${item.packId}/${item.rule.name} may need an executable detector family.`, risk: "medium" });
  }
  return { suggestions, beforeAfterDiff: suggestions.map((suggestion) => suggestion.message), warnings: suggestions.length ? [] : ["No refactor issues detected from supplied packs."] };
}

export function minimizePolicySet(input: { brief?: ProjectBrief; rules?: string[]; tokenBudget?: number } = {}) {
  const text = `${input.brief?.idea ?? ""} ${input.brief?.risk ?? ""} ${Object.values(input.brief?.stack ?? {}).join(" ")}`.toLowerCase();
  const rules = input.rules ?? listFoundationPacks().flatMap((pack) => pack.rules);
  const scored = rules.map((rule) => {
    const usefulness = rule.toLowerCase().split(/\W+/).filter((word) => word.length > 4 && text.includes(word)).length;
    return { rule, usefulness, recommendation: usefulness > 0 ? "keep" : rules.length > 8 ? "defer" : "keep" };
  });
  return {
    rules: scored,
    compactProfile: scored.filter((rule) => rule.recommendation === "keep").slice(0, input.tokenBudget ? Math.max(3, Math.floor(input.tokenBudget / 80)) : 8),
    noiseForecast: scored.filter((rule) => rule.recommendation === "defer").length > 5 ? "high" : "low"
  };
}

export function selectReviewPlaybook(input: { request?: string; brief?: ProjectBrief } = {}) {
  const text = `${input.request ?? ""} ${input.brief?.idea ?? ""}`.toLowerCase();
  const id = /security|auth|secret/.test(text) ? "security-sensitive-change"
    : /dependency|package|config/.test(text) ? "dependency-config-change"
      : /docs|readme|llms|agents/.test(text) ? "documentation-refresh"
        : /refactor/.test(text) ? "refactor"
          : /bug|fix/.test(text) ? "bug-fix"
            : "feature-addition";
  return {
    playbook: {
      id,
      intakeChecks: ["Interpret implementation intent", "Create pre-edit contract for risky scope"],
      policyPacks: ["agent-harness", "testing", "repo-structure"],
      reviewTools: requiredToolsForPlaybook(id),
      proofRequirements: ["Named verification command", "Architecture review gate", "Not-done disclosure"]
    }
  };
}

export function reviewPlaybookConformance(input: { playbookId?: string; toolsRun?: string[]; verification?: string[] } = {}) {
  const required = requiredToolsForPlaybook(input.playbookId);
  const missing = required.filter((tool) => !(input.toolsRun ?? []).includes(tool));
  return {
    valid: missing.length === 0 && (input.verification ?? []).length > 0,
    missing,
    recommendation: missing.length ? `Run missing tools: ${missing.join(", ")}.` : "Playbook conformance looks usable."
  };
}

export function checkAgentCollaborationPlan(input: { ownership?: Array<{ agent: string; files: string[] }>; doNotTouch?: string[] } = {}) {
  const owners = new Map<string, string>();
  const conflicts: string[] = [];
  for (const entry of input.ownership ?? []) {
    for (const file of entry.files) {
      const existing = owners.get(file);
      if (existing && existing !== entry.agent) conflicts.push(`${file} is claimed by ${existing} and ${entry.agent}.`);
      owners.set(file, entry.agent);
      if ((input.doNotTouch ?? []).some((pattern) => file === pattern || matchesPathPattern(file, pattern))) conflicts.push(`${file} violates a do-not-touch boundary.`);
    }
  }
  return {
    valid: conflicts.length === 0,
    conflicts,
    integrationChecklist: ["Confirm file ownership", "Run combined tests", "Review final diff against contracts", "State unresolved risks"]
  };
}

export function runFailureModeDrills(input: { cases?: string[] } = {}) {
  const cases = input.cases ?? ["skipped verification", "fake root cause", "dependency churn", "broad rewrite", "misplaced secrets", "ui server leak", "rule overreach"];
  const drills = cases.map(runFailureModeDrill);
  return {
    status: drills.every((drill) => drill.caught && !drill.escaped) ? "pass" : "fail",
    drills,
    suggestedDetectors: drills.filter((drill) => !drill.caught || drill.escaped).map((drill) => `Add or repair detector for ${drill.name}.`)
  };
}

export function calibrateRuleImpact(input: { findings?: ReviewViolation[]; profile?: "minimal" | "balanced" | "strict" } = {}) {
  const findings = input.findings ?? [];
  return {
    profile: input.profile ?? "balanced",
    severityChanges: findings.filter((finding) => finding.confidence === "low").map((finding) => ({ code: finding.code, recommendation: "consider warning until evidence improves" })),
    confidenceAdjustments: findings.map((finding) => ({ code: finding.code, confidence: finding.confidence, gateImpact: finding.severity === "error" ? "blocks" : "warns" })),
    previewGateChange: findings.some((finding) => finding.severity === "error") ? "strict profiles would fail" : "strict profiles would warn or pass"
  };
}

export function reviewDocumentationIntelligence(input: { readme?: string; llmsTxt?: string; agentsMd?: string; toolNames?: string[] } = {}) {
  const findings: string[] = [];
  if (input.toolNames?.length) {
    if (!input.readme?.trim()) findings.push("README not supplied for changed tool-surface review.");
    if (!input.llmsTxt?.trim()) findings.push("llms.txt not supplied for changed tool-surface review.");
    if (!input.agentsMd?.trim()) findings.push("AGENTS.md not supplied for changed tool-surface review.");
  }
  for (const tool of input.toolNames ?? []) {
    if (input.readme && !input.readme.includes(tool)) findings.push(`README missing ${tool}.`);
    if (input.llmsTxt && !input.llmsTxt.includes(tool)) findings.push(`llms.txt missing ${tool}.`);
  }
  const agentScore = input.agentsMd ? scoreAgentInstructions(input.agentsMd) : undefined;
  const llmsScore = input.llmsTxt ? scoreLlmsTxt(input.llmsTxt) : undefined;
  return {
    status: findings.length ? "warn" : "pass",
    findings,
    agentScore,
    llmsScore,
    releaseNotes: input.toolNames?.length ? `Updated tool surface: ${input.toolNames.join(", ")}.` : "No tool-surface changes supplied."
  };
}

function detectorForDrill(name: string): string {
  if (/verification|root cause/.test(name)) return "final-response-review";
  if (/secret/.test(name)) return "mcp-security-review";
  if (/ui|server|rewrite/.test(name)) return "repo-structure-review";
  return "policy-review";
}

function requiredToolsForPlaybook(playbookId: string | undefined): string[] {
  if (playbookId === "security-sensitive-change") {
    return ["interpret_implementation_intent", "create_pre_edit_contract", "review_implementation_against_contract", "review_agent_final_response"];
  }
  if (playbookId === "refactor" || playbookId === "risky-refactor") {
    return ["interpret_implementation_intent", "create_pre_edit_contract", "review_implementation_against_contract", "review_agent_final_response"];
  }
  if (playbookId === "documentation-refresh") {
    return ["score_agent_artifacts", "review_documentation_intelligence", "review_agent_final_response"];
  }
  return ["interpret_implementation_intent", "review_repo_structure", "review_agent_final_response"];
}

function runFailureModeDrill(name: string) {
  const normalized = name.toLowerCase();
  const detector = detectorForDrill(normalized);
  if (/skipped verification/.test(normalized)) {
    const positive = reviewAgentFinalResponse({ response: "Changed code. Verification skipped. Assumptions: none. Not done: no remaining work.", requiredChecks: ["npm test"] });
    const negative = reviewAgentFinalResponse({ response: "Changed code. Verified with npm test. Assumptions: none. Not done: no remaining work.", requiredChecks: ["npm test"] });
    return drillResult(name, detector, positive.status !== "pass", negative.status === "pass", JSON.stringify({ positive: positive.status, negative: negative.status }));
  }
  if (/fake root cause/.test(normalized)) {
    const positive = reviewAgentFinalResponse({ response: "Changed cache code. Verified with npm test. Root cause: the cache was stale. Assumptions: none. Not done: no remaining work." });
    const negative = reviewAgentFinalResponse({ response: "Changed cache code. Verified with npm test. Root cause evidence: log showed stale cache reads. Assumptions: none. Not done: no remaining work." });
    return drillResult(name, detector, positive.status === "fail", negative.status === "pass", JSON.stringify({ positive: positive.status, negative: negative.status }));
  }
  if (/misplaced secrets/.test(normalized)) {
    const positive = reviewMcpConfigSecurity({ config: { mcpServers: { bad: { command: "npx", args: ["pkg@latest", "--token", "sk-123456789012345678901234"] } } } });
    const negative = reviewMcpConfigSecurity({ config: { mcpServers: { ok: { command: "npx", args: ["pkg@1.2.3"], env: { TOKEN: "${API_TOKEN}" } } } } });
    return drillResult(name, detector, positive.status === "fail", negative.status !== "fail", JSON.stringify({ positive: positive.status, negative: negative.status }));
  }
  if (/ui server leak/.test(normalized)) {
    const positive = reviewFileSummaries([{ path: "src/features/dashboard/Dashboard.tsx", imports: ["@/server/db"], hasUseClient: true }]);
    const negative = reviewFileSummaries([{ path: "src/features/dashboard/Dashboard.tsx", imports: ["@/shared/ui/Button"], hasUseClient: true }]);
    return drillResult(name, detector, positive.some((finding) => finding.severity === "error"), !negative.some((finding) => finding.severity === "error"), JSON.stringify({ positive: positive.map((finding) => finding.code), negative: negative.map((finding) => finding.code) }));
  }
  if (/broad rewrite/.test(normalized)) {
    const positive = reviewFileSummaries([{ path: "src/App.tsx", lines: 900, imports: ["@/features/a", "@/features/b"] }]);
    const negative = reviewFileSummaries([{ path: "src/features/todos/TodoList.tsx", lines: 120, imports: ["@/shared/ui/Button"] }]);
    return drillResult(name, detector, positive.length > 0, negative.length === 0, JSON.stringify({ positive: positive.map((finding) => finding.code), negative: negative.map((finding) => finding.code) }));
  }
  if (/dependency churn/.test(normalized)) {
    const positive = reviewAgentFinalResponse({ response: "Changed package versions. Verified with npm test. Assumptions: none. Not done: no remaining work.", requiredChecks: ["npm audit"] });
    const negative = reviewAgentFinalResponse({ response: "Changed package versions. Verified with npm test and npm audit. Assumptions: none. Not done: no remaining work.", requiredChecks: ["npm audit"] });
    return drillResult(name, detector, positive.status === "fail", negative.status === "pass", JSON.stringify({ positive: positive.status, negative: negative.status }));
  }
  if (/rule overreach/.test(normalized)) {
    const positive = reviewAgentFinalResponse({ response: "Changed rule severity. Verified with npm test. Assumptions: none." });
    const negative = reviewAgentFinalResponse({ response: "Changed rule severity. Verified with npm test. Assumptions: none. Not done: no remaining work." });
    return drillResult(name, detector, positive.status !== "pass", negative.status === "pass", JSON.stringify({ positive: positive.status, negative: negative.status }));
  }
  return {
    name,
    caught: false,
    detector: "unknown",
    escaped: true,
    evidence: "Unknown failure-mode drill case."
  };
}

function drillResult(name: string, detector: string, positiveCaught: boolean, negativeClean: boolean, evidence: string) {
  return {
    name,
    caught: positiveCaught,
    detector,
    escaped: !negativeClean,
    evidence
  };
}
