import { publicSafeSummary } from "./publicSafetyText.js";
import { isFoundryActionabilityReport } from "./foundryDecisionLedgerInput.js";
import type {
  FoundryActionabilityAssessment,
  FoundryActionabilityReport
} from "./foundryActionabilityTypes.js";
import type {
  FoundryDecisionApprovalState,
  FoundryDecisionLedgerEntry,
  FoundryDecisionLedgerInput,
  FoundryDecisionLedgerReport,
  FoundryDecisionRedactionState,
  FoundryDecisionRoute
} from "./foundryDecisionLedgerTypes.js";

export type { FoundryDecisionLedgerInput, FoundryDecisionLedgerReport } from "./foundryDecisionLedgerTypes.js";

export function routeFoundryDecisions(input: FoundryDecisionLedgerInput = {}): FoundryDecisionLedgerReport {
  const actionability = isFoundryActionabilityReport(input.actionability) ? input.actionability : emptyActionability();
  const entries = actionability.assessments.map((assessment, index) => ledgerEntry(assessment, index));
  return {
    schemaVersion: 1,
    ledgerId: "foundry-ledger",
    recordedAt: safeOptionalText(input.recordedAt),
    summary: summarizeEntries(entries),
    entries,
    publicSafety: {
      rawPayloadsIncluded: false,
      rawRepoContentIncluded: false,
      localPathsIncluded: false,
      tokenValuesIncluded: false,
      mutationAllowed: false,
      publicLedgerOnly: true
    }
  };
}

function ledgerEntry(assessment: FoundryActionabilityAssessment, index: number): FoundryDecisionLedgerEntry {
  const route = routeAssessment(assessment);
  const redactionState = redactionStateFor(assessment);
  const approvalState = approvalStateFor(route);
  return {
    id: `fdl-${String(index + 1).padStart(3, "0")}-${route}`,
    route,
    evidenceIds: [`fde-${String(index + 1).padStart(3, "0")}`],
    source: {
      score: clamp(assessment.score)
    },
    decisionReason: decisionReason(route, assessment),
    redactionState,
    verificationRequirements: publicSafeList(assessment.requiredVerification),
    approvalState,
    nextAction: nextActionFor(route),
    mutation: {
      serverMutationAllowed: false,
      explicitApprovalRequired: approvalState === "approval_required"
    }
  };
}

function routeAssessment(assessment: FoundryActionabilityAssessment): FoundryDecisionRoute {
  if (!hasRoutingFactors(assessment)) return "ask_human";
  if (requiresHumanDisclosure(assessment)) return "ask_human";
  if (assessment.decision === "pr_preview_candidate") return "pr_preview";
  if (assessment.decision === "exception_candidate") return "exception";
  if (assessment.decision === "no_op_candidate") return "no_op";
  if (factorScore(assessment, "maintainer_value") >= 55 && assessment.score >= 50) return "architect_issue";
  return "ask_human";
}

function requiresHumanDisclosure(assessment: FoundryActionabilityAssessment): boolean {
  const publicSafety = factorScore(assessment, "public_safety_risk");
  const text = assessmentText(assessment);
  return publicSafety < 50 ||
    hasRawRepoContent(text) ||
    /security-sensitive|redacted evidence|identity fields|safe disclosure|human disclosure/.test(text);
}

function redactionStateFor(assessment: FoundryActionabilityAssessment): FoundryDecisionRedactionState {
  const text = assessmentText(assessment);
  if (factorScore(assessment, "public_safety_risk") < 35 || /security-sensitive|safe disclosure/.test(text)) return "sensitive";
  if (factorScore(assessment, "public_safety_risk") < 75 || hasRawRepoContent(text) || /redacted|identity fields|\[redacted-/.test(text)) return "redacted";
  return "public";
}

function approvalStateFor(route: FoundryDecisionRoute): FoundryDecisionApprovalState {
  if (route === "pr_preview" || route === "architect_issue" || route === "exception") return "approval_required";
  if (route === "ask_human") return "human_required";
  return "not_required";
}

function decisionReason(route: FoundryDecisionRoute, assessment: FoundryActionabilityAssessment): string {
  const blocker = assessment.blockers[0] ? ` Primary blocker: ${assessment.blockers[0]}` : "";
  const rationale = assessment.publicRationale[0] ? ` ${assessment.publicRationale[0]}` : "";
  return publicSafeLedgerSummary(`Route ${route} at score ${clamp(assessment.score)}.${blocker}${rationale}`);
}

function nextActionFor(route: FoundryDecisionRoute): string {
  const actions: Record<FoundryDecisionRoute, string> = {
    pr_preview: "Draft a local PR preview after approval; do not write to GitHub without explicit mutation approval.",
    architect_issue: "Draft an architect-mcp issue preview for maintainer review; do not create the issue without explicit approval.",
    exception: "Record an exception candidate after the suppression prerequisite is reviewed and approved.",
    no_op: "Record the no-op rationale; no repository mutation is needed.",
    ask_human: "Ask a human maintainer to resolve disclosure, verification, or ambiguity before routing."
  };
  return actions[route];
}

function factorScore(assessment: FoundryActionabilityAssessment, name: string): number {
  return assessment.factors.find((factor) => factor.name === name)?.score ?? 0;
}

function hasRoutingFactors(assessment: FoundryActionabilityAssessment): boolean {
  return hasSingleFactor(assessment, "public_safety_risk") &&
    hasSingleFactor(assessment, "maintainer_value");
}

function hasSingleFactor(assessment: FoundryActionabilityAssessment, name: string): boolean {
  return assessment.factors.filter((factor) => factor.name === name).length === 1;
}

function assessmentText(assessment: FoundryActionabilityAssessment): string {
  return [
    ...assessment.blockers,
    ...assessment.publicRationale,
    ...assessment.requiredVerification
  ].join(" ").toLowerCase();
}

function summarizeEntries(entries: FoundryDecisionLedgerEntry[]): FoundryDecisionLedgerReport["summary"] {
  const byRoute = countBy(entries.map((entry) => entry.route));
  return {
    totalEntries: entries.length,
    byRoute,
    approvalRequired: entries.filter((entry) => entry.approvalState === "approval_required").length,
    humanRequired: entries.filter((entry) => entry.approvalState === "human_required").length,
    publicSafetyHolds: entries.filter((entry) => entry.redactionState !== "public").length,
    serverWritesPerformed: 0
  };
}

function publicSafeList(values: string[]): string[] {
  return values.map((value) => publicSafeLedgerSummary(value));
}

function safeOptionalText(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const safe = publicSafeLedgerSummary(value);
  return safe.length > 0 ? safe : undefined;
}

function publicSafeLedgerSummary(value: string): string {
  const safe = publicSafeSummary(value);
  if (hasRawRepoContent(safe.value)) return "[redacted-raw-repo-content]";
  return safe.value;
}

function hasRawRepoContent(value: string): boolean {
  const codeLines = value.split("\n").filter((line) =>
    /^\s*(?:(?:import|export|const|let|var|function|class|interface|type|if|for|while|return|await)\b|[{}]\s*$)/.test(line)
  );
  return codeLines.length >= 2 ||
    /\b(?:import\s+[^;]+?\s+from\s+["']|export\s+(?:const|function|class|type|interface)|(?:const|let|var)\s+\w+\s*=|function\s+\w+\s*\(|class\s+\w+\s*\{|interface\s+\w+\s*\{|type\s+\w+\s*=|process\.env\.)/.test(value);
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

function emptyActionability(): FoundryActionabilityReport {
  return {
    schemaVersion: 1,
    summary: { totalFindings: 0, byDecision: {}, prPreviewCandidates: 0, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 0 },
    assessments: [],
    publicSafety: { rawPayloadsIncluded: false, mutationAllowed: false, publicRecommendationsOnly: true }
  };
}
