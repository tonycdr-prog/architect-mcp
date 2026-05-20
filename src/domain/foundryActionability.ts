import { publicSafeSummary, publicSafeText } from "./publicSafetyText.js";
import { safeAssessmentIdentity } from "./foundryActionabilitySafety.js";
import type {
  FoundryActionabilityAssessment,
  FoundryActionabilityDecision,
  FoundryActionabilityFactor,
  FoundryActionabilityFactorName,
  FoundryActionabilityInput,
  FoundryActionabilityReport
} from "./foundryActionabilityTypes.js";
import type { FoundryEvidenceInventory, FoundryEvidenceItem } from "./foundryEvidenceTypes.js";

export type { FoundryActionabilityInput, FoundryActionabilityReport } from "./foundryActionabilityTypes.js";

export function scoreFoundryActionability(input: FoundryActionabilityInput = {}): FoundryActionabilityReport {
  const inventory = isFoundryEvidenceInventory(input.inventory) ? input.inventory : emptyInventory();
  const findings = inventory.evidence.filter((item) => item.kind === "finding");
  const assessments = findings.map((item) => assessFinding(item, inventory, input.verificationHints ?? []));
  return {
    schemaVersion: 1,
    summary: summarizeAssessments(assessments),
    assessments,
    publicSafety: {
      rawPayloadsIncluded: false,
      mutationAllowed: false,
      publicRecommendationsOnly: true
    }
  };
}

function assessFinding(item: FoundryEvidenceItem, inventory: FoundryEvidenceInventory, hints: string[]): FoundryActionabilityAssessment {
  const identity = safeAssessmentIdentity(item);
  const factors = [
    evidenceStrength(item, inventory),
    factor("confidence", confidenceScore(item), `Evidence confidence is ${item.confidence}.`),
    blastRadius(item),
    patchSize(item),
    maintainerFit(item, inventory),
    duplicateRisk(item, inventory),
    releaseImpact(item),
    verificationPath(inventory, hints),
    publicSafetyRisk(item, identity.redacted),
    maintainerValue(item)
  ];
  const blockers = publicSafeList(decisionBlockers(item, factors, inventory, identity.redacted));
  const score = weightedScore(factors);
  const decision = decide(item, factors, blockers, score);
  return {
    evidenceId: identity.evidenceId,
    sourceType: identity.sourceType,
    code: identity.code,
    path: identity.path,
    decision,
    score,
    publicRationale: publicSafeList(rationaleFor(decision, score, factors, blockers)),
    requiredVerification: requiredVerification(decision, item, inventory, hints),
    blockers,
    factors
  };
}

function evidenceStrength(item: FoundryEvidenceItem, inventory: FoundryEvidenceInventory): FoundryActionabilityFactor {
  const sourceBonus = item.sourceType === "architect_review" ? 8 : item.sourceType === "external_tool" ? 2 : -8;
  const coveragePenalty = inventory.coverage.scanTruncated || inventory.coverage.detailedFindingsTruncated || inventory.coverage.caveats.length > 0 ? 12 : 0;
  const score = confidenceScore(item) + sourceBonus - coveragePenalty - (item.redactionStatus !== "none" ? 8 : 0);
  return factor("evidence_strength", score, "Evidence strength combines source, confidence, coverage completeness, and redaction state.");
}

function blastRadius(item: FoundryEvidenceItem): FoundryActionabilityFactor {
  const text = `${item.path ?? ""} ${item.code ?? ""}`.toLowerCase();
  let score = 70;
  if (!item.path) score = 45;
  if (/docs|examples|readme|\.mdx?$/.test(text)) score = 85;
  if (/test|fixture|testdata/.test(text)) score = 75;
  if (/package|lock|release|changelog|workflow|\.github/.test(text)) score = 45;
  if (/^(?:src\/)?(?:index|main|server|app|route)\./i.test(item.path ?? "")) score = 45;
  return factor("blast_radius", score, "Lower-blast findings are better candidates for maintainer-native PR previews.");
}

function patchSize(item: FoundryEvidenceItem): FoundryActionabilityFactor {
  let score = 65;
  if (item.suppressionCandidate) score = 30;
  if (/ARCH001_OVERSIZED_FILE/.test(item.code ?? "")) score = 35;
  if (/ARCH005_MISSING_DIRECTORY|MISSING_PR_TEMPLATE/.test(item.code ?? "")) score = 70;
  if (/docs|examples|readme|\.mdx?$/i.test(item.path ?? "")) score = 80;
  return factor("patch_size", score, "Patch-size confidence is inferred from finding code, path, and suppression status.");
}

function maintainerFit(item: FoundryEvidenceItem, inventory: FoundryEvidenceInventory): FoundryActionabilityFactor {
  const hasTemplate = inventory.evidence.some((entry) => entry.code === "repo_constitution:pr_template");
  let score = hasTemplate ? 75 : 55;
  if (item.sourceType === "repo_constitution") score -= 10;
  if (item.suppressionCandidate) score = Math.min(score, 35);
  return factor("maintainer_fit", score, "Maintainer fit prefers repo-template evidence and penalizes prerequisite suppression categories.");
}

function duplicateRisk(item: FoundryEvidenceItem, inventory: FoundryEvidenceInventory): FoundryActionabilityFactor {
  const histogramCount = inventory.coverage.findingHistogram.find((entry) => entry.code === item.code)?.count ?? 1;
  let score = histogramCount > 10 ? 45 : 80;
  if (item.confidence === "low") score = Math.min(score, 45);
  if (item.suppressionCandidate) score = Math.min(score, 35);
  return factor("duplicate_risk", score, "Duplicate risk increases when similar findings dominate coverage or look suppressible.");
}

function releaseImpact(item: FoundryEvidenceItem): FoundryActionabilityFactor {
  const text = `${item.path ?? ""} ${item.publicSummary} ${item.recommendation ?? ""}`.toLowerCase();
  const score = /release|changelog|version|package|lockfile|workflow|publish|deploy/.test(text) ? 45 : 85;
  return factor("release_impact", score, "Release-sensitive evidence should not become a PR preview without stronger verification.");
}

function verificationPath(inventory: FoundryEvidenceInventory, hints: string[]): FoundryActionabilityFactor {
  const checks = verificationItems(inventory);
  if (checks.some((item) => item.code === "verification:failed")) return factor("verification_path", 20, "A recorded verification check failed.");
  const passedChecks = checks.filter((item) => item.code === "verification:passed").map((item) => safeVerificationLabel(verificationSourceId(item)));
  if (passedChecks.some((item) => item.redacted)) return factor("verification_path", 45, "A passing verification record needs a public-safe label before routing.");
  if (passedChecks.length > 0) return factor("verification_path", 85, "A passing verification record is present.");
  return factor("verification_path", 45, hints.length > 0 ? "Verification hints are available but not yet passed." : "No passing verification evidence is present.");
}

function publicSafetyRisk(item: FoundryEvidenceItem, identityRedacted: boolean): FoundryActionabilityFactor {
  if (item.publicSafetyClass === "sensitive") return factor("public_safety_risk", 10, "Sensitive evidence requires human disclosure review.");
  if (item.redactionStatus !== "none" || identityRedacted) return factor("public_safety_risk", 45, "Evidence is redacted or raw payloads were omitted.");
  return factor("public_safety_risk", 85, "Evidence is public-safe and does not include raw payloads.");
}

function maintainerValue(item: FoundryEvidenceItem): FoundryActionabilityFactor {
  let score = item.severity === "error" ? 85 : item.severity === "warning" ? 65 : 40;
  if (item.suppressionCandidate) score -= 20;
  if (item.sourceType === "repo_constitution" && /MISSING_AGENT_INSTRUCTIONS/.test(item.code ?? "")) score = 35;
  return factor("maintainer_value", score, "Expected maintainer value is inferred from severity, ownership fit, and suppression likelihood.");
}

function decide(item: FoundryEvidenceItem, factors: FoundryActionabilityFactor[], blockers: string[], score: number): FoundryActionabilityDecision {
  if (item.publicSafetyClass === "sensitive" || factorScore(factors, "public_safety_risk") < 50) return "ask_human";
  if (item.suppressionCandidate) return "exception_candidate";
  if (factorScore(factors, "confidence") < 50 || factorScore(factors, "evidence_strength") < 50) return "ask_human";
  if (factorScore(factors, "maintainer_value") < 45) return "no_op_candidate";
  if (blockers.length === 0 && score >= 70 && factorScore(factors, "blast_radius") >= 60 && factorScore(factors, "verification_path") >= 50) return "pr_preview_candidate";
  return score < 45 ? "no_op_candidate" : "ask_human";
}

function decisionBlockers(item: FoundryEvidenceItem, factors: FoundryActionabilityFactor[], inventory: FoundryEvidenceInventory, identityRedacted: boolean): string[] {
  const blockers: string[] = [];
  if (item.publicSafetyClass === "sensitive") blockers.push("Security-sensitive evidence needs human disclosure review before public recommendations.");
  if (item.redactionStatus !== "none") blockers.push("Redacted evidence needs human review before public PR or issue text.");
  if (identityRedacted) blockers.push("Caller-supplied evidence identity fields needed redaction before public routing.");
  if (item.suppressionCandidate) blockers.push(`Suppression prerequisite ${item.suppressionCandidate.category} should be resolved before routing.`);
  if (factorScore(factors, "verification_path") < 50) blockers.push("A passing verification path is missing.");
  if (inventory.coverage.scanTruncated) blockers.push("Audit coverage is partial because the file scan was truncated.");
  return blockers;
}

function rationaleFor(decision: FoundryActionabilityDecision, score: number, factors: FoundryActionabilityFactor[], blockers: string[]): string[] {
  const weakest = [...factors].sort((a, b) => a.score - b.score).slice(0, 2).map((item) => `${item.name}: ${item.rationale}`);
  return [`Decision ${decision} with score ${score}.`, ...blockers.slice(0, 2), ...weakest];
}

function requiredVerification(decision: FoundryActionabilityDecision, item: FoundryEvidenceItem, inventory: FoundryEvidenceInventory, hints: string[]): string[] {
  const passed = verificationItems(inventory).filter((entry) => entry.code === "verification:passed").map((entry) => safeVerificationLabel(verificationSourceId(entry)).value);
  const checks = passed.length > 0 ? passed : hints;
  const base = checks.length > 0 ? checks.map((check) => `Re-run ${check} before mutation approval.`) : ["Record a passing repo verification check before mutation approval."];
  if (decision === "ask_human" && item.publicSafetyClass === "sensitive") base.unshift("Confirm safe disclosure boundaries with a human maintainer.");
  return publicSafeList(base);
}

function weightedScore(factors: FoundryActionabilityFactor[]): number {
  const weights: Record<FoundryActionabilityFactorName, number> = {
    evidence_strength: 1.3,
    confidence: 1,
    blast_radius: 1,
    patch_size: 0.8,
    maintainer_fit: 1,
    duplicate_risk: 0.8,
    release_impact: 0.8,
    verification_path: 1.1,
    public_safety_risk: 1.3,
    maintainer_value: 1.2
  };
  const totalWeight = factors.reduce((sum, item) => sum + weights[item.name], 0);
  return Math.round(factors.reduce((sum, item) => sum + item.score * weights[item.name], 0) / totalWeight);
}

function factor(name: FoundryActionabilityFactorName, rawScore: number, rationale: string): FoundryActionabilityFactor {
  const score = clamp(rawScore);
  return {
    name,
    score,
    status: score >= 75 ? "strong" : score >= 55 ? "mixed" : score >= 35 ? "weak" : "blocked",
    rationale: publicSafeSummary(rationale).value
  };
}

function confidenceScore(item: FoundryEvidenceItem): number {
  return item.confidence === "high" ? 90 : item.confidence === "medium" ? 65 : 35;
}

function factorScore(factors: FoundryActionabilityFactor[], name: FoundryActionabilityFactorName): number {
  return factors.find((factorItem) => factorItem.name === name)?.score ?? 0;
}

function verificationItems(inventory: FoundryEvidenceInventory): FoundryEvidenceItem[] {
  return inventory.evidence.filter((item) => item.kind === "verification");
}

function verificationSourceId(item: FoundryEvidenceItem): string {
  return typeof item.sourceRef?.sourceId === "string" ? item.sourceRef.sourceId : "recorded verification";
}

function safeVerificationLabel(value: string): { value: string; redacted: boolean } {
  const safe = publicSafeSummary(value);
  return { value: safe.value.length > 0 ? safe.value : "recorded verification", redacted: safe.redacted };
}

function publicSafeList(values: string[]): string[] {
  return values.map((value) => publicSafeSummary(value).value);
}

function summarizeAssessments(assessments: FoundryActionabilityAssessment[]): FoundryActionabilityReport["summary"] {
  const byDecision = countBy(assessments.map((assessment) => assessment.decision));
  return {
    totalFindings: assessments.length,
    byDecision,
    prPreviewCandidates: byDecision.pr_preview_candidate ?? 0,
    askHuman: byDecision.ask_human ?? 0,
    exceptionCandidates: byDecision.exception_candidate ?? 0,
    noOpCandidates: byDecision.no_op_candidate ?? 0,
    publicSafetyHolds: assessments.filter((item) => item.factors.find((factorItem) => factorItem.name === "public_safety_risk" && factorItem.score < 50)).length
  };
}

function countBy(values: string[]): Record<string, number> {
  return values.reduce<Record<string, number>>((acc, value) => {
    acc[value] = (acc[value] ?? 0) + 1;
    return acc;
  }, {});
}

function clamp(score: number): number {
  return Math.max(0, Math.min(100, Math.round(score)));
}

function isFoundryEvidenceInventory(value: unknown): value is FoundryEvidenceInventory {
  return isRecord(value) && value.schemaVersion === 1 && Array.isArray(value.evidence) && isRecord(value.coverage);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function emptyInventory(): FoundryEvidenceInventory {
  return {
    schemaVersion: 1,
    summary: { totalEvidence: 0, bySourceType: {}, byConfidence: {}, byPublicSafetyClass: {}, redacted: 0, omittedRawPayloads: 0, suppressionCandidates: 0, coverageCaveats: 0 },
    evidence: [],
    coverage: { scanTruncated: false, detailedFindingsTruncated: false, topScannedDirectories: [], findingHistogram: [], caveats: [] },
    suppressionPrerequisites: [],
    publicSafety: { rawPayloadsIncluded: false, rawRepoContentIncluded: false, mutationAllowed: false }
  };
}
