import { listFoundationPacks } from "./foundationPacks.js";
import { listStackPacks } from "./stackPacks.js";
import { createReviewReport } from "./reviewReport.js";
import type { FileRule, ProjectBrief, ReviewReport, ReviewViolation, StackPack } from "./types.js";

export function createArchitectureStrategyMap(input: { brief?: ProjectBrief; stackPacks?: StackPack[] } = {}) {
  const packs = input.stackPacks ?? listStackPacks();
  const risks = riskSignals(input.brief);
  return {
    goals: input.brief?.coreFlows ?? [input.brief?.idea ?? "Project goal not supplied."],
    risks,
    standards: packs.map((pack) => ({ id: pack.id, rules: pack.fileRules.length, boundaries: pack.moduleBoundaries })),
    riskToStandard: risks.map((risk) => ({ risk, standards: packs.filter((pack) => JSON.stringify(pack).toLowerCase().includes(risk)).map((pack) => pack.id) })),
    verificationToRisk: (input.brief?.verification ?? ["Run focused tests", "Run architecture review"]).map((check) => ({ check, risks })),
    gaps: risks.filter((risk) => !packs.some((pack) => JSON.stringify(pack).toLowerCase().includes(risk)))
  };
}

export function compareStandardsProfiles() {
  return {
    profiles: [
      { profile: "minimal", catches: ["monolith drift", "missing verification", "unsafe client/server leaks"], allows: ["manual review for lower-risk quality rules"] },
      { profile: "balanced", catches: ["architecture boundaries", "repo hygiene", "stack-pack rules", "verification gaps"], allows: ["warnings for noisy migration debt"] },
      { profile: "strict", catches: ["all error findings", "high-risk dependency and security drift", "missing proof"], allows: ["documented accepted baselines only"] }
    ],
    nonNegotiable: ["secrets", "destructive commands", "auth/payment boundary changes", "evidence-before-completion"],
    focusedQuestion: "Do you want speed-oriented minimal checks or strict production gates for this change?"
  };
}

export function previewChangeWhatIf(input: { files?: Array<{ path: string; purpose?: string; responsibilities?: string[] }>; profile?: "minimal" | "balanced" | "strict"; findings?: ReviewViolation[] } = {}) {
  const fileText = (input.files ?? []).map((file) => `${file.path} ${file.purpose ?? ""} ${(file.responsibilities ?? []).join(" ")}`).join(" ");
  const highRisk = /auth|payment|database|migration|security|api|delete|dependency/i.test(fileText);
  const report = createReviewReport(input.findings ?? [], { mode: input.profile === "strict" ? "strict" : "summary" });
  return {
    blastRadius: highRisk ? "high" : (input.files?.length ?? 0) > 6 ? "medium" : "low",
    likelyReview: report,
    requiredVerification: highRisk ? ["Run tests", "Run architecture review", "State rollback plan"] : ["Run focused verification", "Run architecture review if files changed"],
    readyToEdit: !highRisk || input.profile !== "minimal",
    warnings: highRisk ? ["High-risk proposed files should have a pre-edit contract and confirmation."] : []
  };
}

export function diagnoseAgentBehavior(input: { sessionSummaries?: string[]; reports?: ReviewReport[] } = {}) {
  const text = `${(input.sessionSummaries ?? []).join("\n")} ${(input.reports ?? []).map((report) => report.gate.reason).join("\n")}`.toLowerCase();
  const patterns = [
    ["over-broad edits", /broad|monolith|too many|unrelated/],
    ["skipped verification", /not run|skipped|missing verification|no proof/],
    ["premature root-cause claims", /root cause|fixed because|must have/],
    ["ignored contracts", /contract|plan|drift|outside/]
  ].filter(([, pattern]) => (pattern as RegExp).test(text)).map(([pattern]) => pattern as string);
  return {
    score: Math.max(0, 100 - patterns.length * 20),
    patterns,
    steeringText: patterns.length ? `Watch for ${patterns.join(", ")}. Re-run pre-edit and verification gates before continuing.` : "Agent behavior looks aligned with the harness loop.",
    handoff: patterns.length ? `Next agent: check ${patterns[0]} first.` : "Next agent: continue with normal contract and verification loop."
  };
}

export function draftRuleCandidate(input: { sourceText?: string; finding?: ReviewViolation; examples?: string[] } = {}) {
  const text = `${input.sourceText ?? ""} ${input.finding?.message ?? ""} ${(input.examples ?? []).join(" ")}`;
  const executable = /import|env|line|client|server|database|test|route|auth|payment/i.test(text);
  const tooVague = text.length < 40 || /best practices|properly|clean/i.test(text);
  return {
    candidate: {
      name: input.finding?.code ?? "candidate-rule",
      rule: input.finding?.recommendation ?? input.sourceText ?? "Rule needs source text.",
      triggerKind: executable ? "manual-or-executable" : "manual-review",
      severity: /security|auth|payment|secret/i.test(text) ? "error" : "warning"
    },
    classification: tooVague ? "too-vague" : executable ? "executable" : "manual-only",
    fixturePairs: ["positive fixture showing violation", "negative fixture showing allowed pattern"],
    warnings: tooVague ? ["Rule needs more concrete evidence and examples before promotion."] : []
  };
}

export function compareReviewTrends(input: { before?: ReviewReport; after?: ReviewReport } = {}) {
  const beforeCodes = new Map((input.before?.violations ?? []).map((finding) => [finding.code + finding.path, finding]));
  const afterCodes = new Map((input.after?.violations ?? []).map((finding) => [finding.code + finding.path, finding]));
  return {
    newFindings: [...afterCodes.entries()].filter(([key]) => !beforeCodes.has(key)).map(([, finding]) => finding),
    resolvedFindings: [...beforeCodes.entries()].filter(([key]) => !afterCodes.has(key)).map(([, finding]) => finding),
    scoreDelta: (input.after?.score ?? 0) - (input.before?.score ?? 0),
    verificationHonestyDelta: (input.after?.gate.status ?? "fail") === "pass" ? "improved-or-stable" : "needs-review",
    suggestedNextFixture: "Add a before/after fixture for the most frequent unresolved finding."
  };
}

export function renderGovernancePack(input: { stackPack?: StackPack; fileRule?: FileRule } = {}) {
  const pack = input.stackPack;
  const rules = pack?.fileRules ?? (input.fileRule ? [input.fileRule] : []);
  return {
    markdown: [
      `# ${pack?.name ?? "Governance Pack"}`,
      "",
      pack?.rationale ?? "Human-readable governance pack rendering.",
      "",
      "## Rules",
      ...rules.map((rule) => `- **${rule.name}**: ${rule.rule}`)
    ].join("\n"),
    recommendation: rules.length ? "adopt now" : "trial",
    checklist: ["Rules have examples", "Detectors are mapped", "Evidence is documented", "Verification is named"],
    foundations: listFoundationPacks().map((pack) => pack.id)
  };
}

function riskSignals(brief?: ProjectBrief): string[] {
  const text = `${brief?.idea ?? ""} ${brief?.risk ?? ""} ${Object.values(brief?.stack ?? {}).join(" ")}`.toLowerCase();
  return ["auth", "payment", "database", "security", "frontend", "backend", "testing"].filter((risk) => text.includes(risk));
}
