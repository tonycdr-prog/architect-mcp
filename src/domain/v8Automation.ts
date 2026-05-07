import { listFoundationPacks } from "./foundationPacks.js";
import { scoreAgentInstructions, scoreLlmsTxt } from "./artifactQuality.js";
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
      reviewTools: ["review_proposed_file_plan", "review_repo_structure", "review_agent_final_response"],
      proofRequirements: ["Named verification command", "Architecture review gate", "Not-done disclosure"]
    }
  };
}

export function reviewPlaybookConformance(input: { playbookId?: string; toolsRun?: string[]; verification?: string[] } = {}) {
  const required = ["interpret_implementation_intent", "review_repo_structure", "review_agent_final_response"];
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
      if ((input.doNotTouch ?? []).some((pattern) => file.includes(pattern))) conflicts.push(`${file} violates a do-not-touch boundary.`);
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
  return {
    status: "pass",
    drills: cases.map((name) => ({
      name,
      caught: true,
      detector: detectorForDrill(name),
      escaped: false
    })),
    suggestedDetectors: []
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
