import { diffArchitectureContracts } from "./contractDiff.js";
import { listFoundationPacks } from "./foundationPacks.js";
import { listRepoProfiles } from "./repoProfiles.js";
import { createReviewReport } from "./reviewReport.js";
import { analyzeStackPackConflicts } from "./stackPackConflicts.js";
import { listStackPacks, resolveStackPacks } from "./stackPacks.js";
import type { ArchitectureContract, FileSummary, ProjectBrief, ReviewBaseline, ReviewGateOptions, ReviewMode, ReviewReport, ReviewViolation, StackPack } from "./types.js";

export type StandardsProfile = "minimal" | "balanced" | "strict";

export type StandardsProfileInput = {
  brief?: ProjectBrief;
  profile?: StandardsProfile;
  stackPackIds?: string[];
  files?: FileSummary[];
};

export function resolveStandardsProfile(input: StandardsProfileInput = {}) {
  const profile = input.profile ?? inferProfile(input);
  const foundationPacks = listFoundationPacks();
  const stackPacks = input.brief ? resolveStackPacks(input.brief.stack ?? {}, input.stackPackIds ?? []) : listStackPacks().filter((pack) => input.stackPackIds?.includes(pack.id));
  const selectedRules = [
    ...foundationPacks.flatMap((pack) => pack.rules.map((rule) => ({ source: "foundation" as const, packId: pack.id, rule }))),
    ...stackPacks.flatMap((pack) => pack.fileRules.map((rule) => ({ source: "stack" as const, packId: pack.id, rule: rule.rule, severity: rule.severity })))
  ];

  return {
    profile,
    selection: {
      foundationPacks: foundationPacks.map((pack) => ({ id: pack.id, name: pack.name, reason: pack.rationale })),
      stackPacks: stackPacks.map((pack) => ({ id: pack.id, name: pack.name, reason: reasonForStackPack(pack, input.brief) })),
      selectedRules
    },
    coverage: {
      foundations: foundationPacks.length,
      stackPacks: stackPacks.length,
      rules: selectedRules.length,
      filesConsidered: input.files?.length ?? 0,
      stackSignals: Object.values(input.brief?.stack ?? {}).filter(Boolean)
    },
    guidance: guidanceForProfile(profile),
    warnings: stackPacks.length === 0 ? ["No stack packs selected; results rely on foundation rules only."] : []
  };
}

export function explainReviewFindings(input: { findings?: ReviewViolation[]; report?: ReviewReport; context?: string } = {}) {
  const findings = input.findings ?? input.report?.violations ?? input.report?.priorityFindings ?? [];
  return {
    count: findings.length,
    explanations: findings.map((finding) => ({
      ...finding,
      plainEnglish: plainEnglishForFinding(finding),
      whatThisProbablyMeans: meaningForFinding(finding),
      fixShape: fixShapeForFinding(finding),
      suggestedNextTool: nextToolForFinding(finding),
      proveItWith: proofForFinding(finding),
      evidence: evidenceForFinding(finding, input.context)
    }))
  };
}

export function simulatePolicyGate(input: { findings?: ReviewViolation[]; report?: ReviewReport; baseline?: ReviewBaseline; customGate?: ReviewGateOptions } = {}) {
  const findings = input.findings ?? input.report?.violations ?? [];
  const modes: ReviewMode[] = ["summary", "migration", "ci", "strict"];
  return {
    simulations: modes.map((mode) => ({
      mode,
      report: createReviewReport(findings, { mode, baseline: input.baseline, gate: input.customGate })
    })),
    custom: input.customGate ? createReviewReport(findings, { mode: "strict", baseline: input.baseline, gate: input.customGate }) : undefined
  };
}

export function analyzeStandardsConflicts(input: { stackPacks?: StackPack[] } = {}) {
  const stackPacks = input.stackPacks ?? listStackPacks();
  const conflicts = analyzeStackPackConflicts(stackPacks);
  return {
    conflicts: conflicts.map((conflict) => ({
      ...conflict,
      rank: conflict.severity === "error" ? "high" : "medium",
      suggestedResolution: conflict.resolution
    })),
    summary: {
      total: conflicts.length,
      high: conflicts.filter((conflict) => conflict.severity === "error").length,
      medium: conflicts.filter((conflict) => conflict.severity === "warning").length
    }
  };
}

export function scoreRepoProfileFit(input: { brief?: ProjectBrief; files?: FileSummary[] } = {}) {
  const files = input.files ?? [];
  const paths = files.map((file) => file.path);
  const scored = listRepoProfiles().map((profile) => {
    const stackScore = Object.values(profile.stack).filter(Boolean).filter((value) => Object.values(input.brief?.stack ?? {}).join(" ").toLowerCase().includes((value ?? "").split(" ")[0].toLowerCase())).length;
    const pathPatterns = Object.values(profile.repoLayout.pathMap).flat();
    const pathScore = pathPatterns.filter((pattern) => paths.some((path) => path.includes(pattern))).length;
    const score = Math.min(100, stackScore * 25 + pathScore * 10);
    return {
      id: profile.id,
      name: profile.name,
      score,
      confidence: score >= 60 ? "high" : score >= 30 ? "medium" : "low",
      suggestedPathMap: profile.repoLayout.pathMap,
      warnings: score < 30 ? ["Profile has weak evidence from supplied stack and file paths."] : []
    };
  }).sort((left, right) => right.score - left.score);

  return {
    best: scored[0],
    profiles: scored
  };
}

export function reviewContractLifecycle(input: { before?: ArchitectureContract; after: ArchitectureContract; findings?: ReviewViolation[] }) {
  const diff = input.before ? diffArchitectureContracts(input.before, input.after) : undefined;
  const errorRules = input.after.fileRules.filter((rule) => rule.severity === "error").length;
  const maturity = errorRules > 8 && input.after.testingExpectations.length > 2 ? "strict" : errorRules > 3 ? "recommended" : "experimental";
  const noisy = (input.findings ?? []).length > 25;
  return {
    maturity,
    diff,
    changelog: diff?.changes.map((change) => change.message) ?? ["Initial contract lifecycle review."],
    deprecations: input.after.fileRules.filter((rule) => /deprecated|replace/i.test(`${rule.name} ${rule.rule}`)).map((rule) => ({
      rule: rule.name,
      replacement: rule.recommendation ?? "Add an explicit replacement rule before removing this rule."
    })),
    noiseRisk: noisy ? "high" : errorRules > 10 ? "medium" : "low",
    warnings: noisy ? ["Current finding volume is high; trial stricter rules before blocking on them."] : []
  };
}

function inferProfile(input: StandardsProfileInput): StandardsProfile {
  const text = `${input.brief?.idea ?? ""} ${input.brief?.risk ?? ""} ${input.brief?.enforcement ?? ""} ${Object.values(input.brief?.stack ?? {}).join(" ")}`.toLowerCase();
  if (/strict|production|security|auth|payment|enterprise|regulated/.test(text)) return "strict";
  if ((input.files?.length ?? 0) <= 8 && /small|prototype|poc|demo/.test(text)) return "minimal";
  return "balanced";
}

function reasonForStackPack(pack: StackPack, brief?: ProjectBrief): string {
  const stack = Object.values(brief?.stack ?? {}).filter(Boolean).join(", ");
  return stack ? `${pack.name} matches requested stack signals: ${stack}.` : `${pack.name} was explicitly selected or available for review.`;
}

function guidanceForProfile(profile: StandardsProfile): string[] {
  if (profile === "minimal") return ["Prefer the fewest rules needed to prevent monolith drift, unsafe boundaries, and skipped verification."];
  if (profile === "strict") return ["Treat high-risk boundaries, missing evidence, dependency churn, and public API drift as blockers."];
  return ["Balance speed with boundary checks, focused verification, and explainable findings."];
}

function plainEnglishForFinding(finding: ReviewViolation): string {
  if (/OVERSIZED|MONOLITH|MIXED/.test(finding.code)) return "A file or plan is carrying too many responsibilities.";
  if (/DB|ENV|CLIENT_SERVER|TRANSPORT|IMPORT_GRAPH/.test(finding.code)) return "Code appears to cross an architecture boundary that should stay separated.";
  if (/VERIFICATION|HARNESS|FINAL/.test(finding.code)) return "The agent has not provided enough proof for the claimed change.";
  return finding.message;
}

function meaningForFinding(finding: ReviewViolation): string {
  if (/auth/i.test(finding.message)) return "This likely affects authentication, authorization, or session ownership.";
  if (/database|db|schema|migration/i.test(finding.message)) return "This likely affects data ownership or server-only database boundaries.";
  if (/dependency|package|upgrade/i.test(finding.message)) return "This likely affects dependency compatibility or rollback risk.";
  return "This is an architecture or workflow rule that needs either a focused fix, a documented exception, or verification evidence.";
}

function fixShapeForFinding(finding: ReviewViolation): string {
  if (/OVERSIZED|MIXED/.test(finding.code)) return "split file";
  if (/MISSING|BOUNDARY|LEAK|DB|ENV/.test(finding.code)) return "add boundary";
  if (/VERIFICATION|HARNESS/.test(finding.code)) return "verify";
  return "review";
}

function nextToolForFinding(finding: ReviewViolation): string {
  if (/PLAN/.test(finding.code)) return "review_proposed_file_plan";
  if (/HARNESS/.test(finding.code)) return "review_implementation_against_contract";
  if (/SPEC|MISSING/.test(finding.code)) return "grill_me";
  return "review_repo_structure";
}

function proofForFinding(finding: ReviewViolation): string {
  if (/test|verification|harness/i.test(finding.message)) return "Run the named verification command and include pass/fail status.";
  if (finding.path) return `Re-run review for ${finding.path} and show the finding is gone or accepted with a reason.`;
  return "Run the relevant review tool and include the resulting gate status.";
}

function evidenceForFinding(finding: ReviewViolation, context?: string): string[] {
  return [finding.path, finding.message, context].filter(Boolean) as string[];
}
