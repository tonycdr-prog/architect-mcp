import { publicSafeSummary, publicSafeText } from "./publicSafetyText.js";
import { summarizeFoundryEvidenceCoverage } from "./foundryEvidenceCoverage.js";
import type {
  FoundryEvidenceInventory,
  FoundryEvidenceInventoryInput,
  FoundryEvidenceItem,
  FoundryEvidenceKind,
  FoundryEvidenceSourceRef,
  FoundryExternalFinding,
  FoundryPublicSafetyClass,
  FoundryRedactionStatus,
  FoundrySuppressionCandidate,
  FoundrySuppressionCategory,
  FoundrySuppressionPrerequisite,
  FoundryVerificationEvidence
} from "./foundryEvidenceTypes.js";
import type { RepoConstitution, RepoConstitutionFinding } from "./repoConstitutionTypes.js";
import type { ReviewReport, ReviewViolation } from "./types.js";

export type { FoundryEvidenceInventory, FoundryEvidenceInventoryInput, FoundryEvidenceItem, FoundryExternalFinding, FoundryVerificationEvidence } from "./foundryEvidenceTypes.js";

export function normalizeFoundryEvidence(input: FoundryEvidenceInventoryInput = {}): FoundryEvidenceInventory {
  const items: FoundryEvidenceItem[] = [];
  for (const [reportIndex, report] of (input.reviewReports ?? []).entries()) {
    collectReviewReportEvidence(items, report, `review-report-${reportIndex + 1}`);
  }
  for (const [index, finding] of (input.findings ?? []).entries()) {
    items.push(reviewFindingEvidence(finding, {
      sourceType: "architect_review",
      sourceId: "supplied-findings",
      index
    }));
  }
  for (const [index, finding] of (input.externalFindings ?? []).entries()) {
    items.push(externalFindingEvidence(finding, index));
  }
  for (const [index, verification] of (input.verification ?? []).entries()) {
    items.push(verificationEvidence(verification, index));
  }
  if (isRepoConstitutionEvidenceInput(input.repoConstitution)) collectRepoConstitutionEvidence(items, input.repoConstitution);
  return {
    schemaVersion: 1,
    summary: summarizeItems(items),
    evidence: items.map((item, index) => ({ ...item, id: `fev-${String(index + 1).padStart(4, "0")}` })),
    coverage: summarizeFoundryEvidenceCoverage(input.reviewReports ?? []),
    suppressionPrerequisites: foundrySuppressionPrerequisites(),
    publicSafety: { rawPayloadsIncluded: false, rawRepoContentIncluded: false, mutationAllowed: false }
  };
}

export function foundrySuppressionPrerequisites(): FoundrySuppressionPrerequisite[] {
  return [
    { category: "generated_file", issue: "https://github.com/tonycdr-prog/architect-mcp/issues/313", reason: "Generated and data-file oversized warnings should be suppressible before actionability scoring." },
    { category: "vendored_code", issue: "https://github.com/tonycdr-prog/architect-mcp/issues/313", reason: "Vendored code is usually not maintainer-owned evidence for a repo-native PR." },
    { category: "fixture_or_test_data", issue: "https://github.com/tonycdr-prog/architect-mcp/issues/313", reason: "Fixture and test data noise should not look like source maintainability debt." },
    { category: "docs_example", issue: "https://github.com/tonycdr-prog/architect-mcp/issues/317", reason: "Documentation examples need repo-type-aware handling before routing." },
    { category: "conventional_entrypoint", issue: "https://github.com/tonycdr-prog/architect-mcp/issues/315", reason: "Conventional package entrypoints should not be treated as hygiene drift by default." },
    { category: "repo_profile_mismatch", issue: "https://github.com/tonycdr-prog/architect-mcp/issues/319", reason: "Language and framework repo profiles are needed before scoring profile-specific findings." }
  ];
}

function collectReviewReportEvidence(items: FoundryEvidenceItem[], report: ReviewReport, sourceId: string): void {
  for (const [index, finding] of uniqueReviewFindings([...(report.priorityFindings ?? []), ...(report.violations ?? [])]).entries()) {
    items.push(reviewFindingEvidence(finding, { sourceType: "architect_review", sourceId, index }));
  }
  for (const [index, caveat] of (report.coverage?.caveats ?? []).entries()) {
    items.push(safeItem({
      kind: "coverage_caveat",
      sourceType: "coverage",
      sourceRef: { sourceType: "coverage", sourceId, index },
      confidence: "high",
      severity: "warning",
      code: "COVERAGE_CAVEAT",
      publicSummary: caveat,
      recommendation: "Treat actionability decisions as partial until coverage caveats are resolved."
    }));
  }
}

function reviewFindingEvidence(finding: ReviewViolation, sourceRef: FoundryEvidenceSourceRef): FoundryEvidenceItem {
  return safeItem({
    kind: "finding",
    sourceType: "architect_review",
    sourceRef,
    confidence: finding.confidence,
    severity: finding.severity,
    code: finding.code,
    path: finding.path,
    publicSummary: finding.message,
    recommendation: finding.recommendation,
    suppressionCandidate: suppressionCandidateForPath(finding.path, finding.code)
  });
}

function uniqueReviewFindings(findings: ReviewViolation[]): ReviewViolation[] {
  const seen = new Set<string>();
  return findings.filter((finding) => {
    const key = `${finding.code}:${finding.path ?? ""}:${finding.message}`;
    return seen.has(key) ? false : (seen.add(key), true);
  });
}

function externalFindingEvidence(finding: FoundryExternalFinding, index: number): FoundryEvidenceItem {
  const code = finding.ruleId ?? `${finding.toolName}:finding`;
  return safeItem({
    kind: "finding",
    sourceType: "external_tool",
    sourceRef: { sourceType: "external_tool", sourceId: finding.toolName, index, path: finding.path },
    confidence: finding.confidence ?? "medium",
    severity: finding.severity ?? "warning",
    code,
    path: finding.path,
    publicSummary: finding.message,
    recommendation: finding.recommendation,
    sensitive: finding.securitySensitive,
    rawPayloadOmitted: finding.rawPayload !== undefined,
    suppressionCandidate: suppressionCandidateForPath(finding.path, code)
  });
}

function verificationEvidence(verification: FoundryVerificationEvidence, index: number): FoundryEvidenceItem {
  const statusSeverity = verification.status === "failed" ? "error" : verification.status === "passed" ? "info" : "warning";
  return safeItem({
    kind: "verification",
    sourceType: "verification",
    sourceRef: { sourceType: "verification", sourceId: verification.check, index },
    confidence: verification.recordedAt || verification.status === "passed" || verification.status === "failed" ? "high" : "medium",
    severity: statusSeverity,
    code: `verification:${verification.status}`,
    publicSummary: `${verification.check}: ${verification.status}${verification.summary ? ` - ${verification.summary}` : ""}`,
    recommendation: verification.status === "passed" ? "Use as supporting verification evidence." : "Do not route a PR preview until verification is passed or explicitly scoped."
  });
}

function collectRepoConstitutionEvidence(items: FoundryEvidenceItem[], constitution: RepoConstitution): void {
  for (const [index, finding] of constitution.findings.entries()) {
    items.push(repoConstitutionFindingEvidence(finding, index));
  }
  for (const [index, template] of constitution.pullRequests.templates.entries()) {
    items.push(safeItem({
      kind: "repo_signal",
      sourceType: "repo_constitution",
      sourceRef: { sourceType: "repo_constitution", sourceId: "pull-request-template", index, path: template.path },
      confidence: template.contentProvided ? "high" : "medium",
      severity: "info",
      code: "repo_constitution:pr_template",
      path: template.path,
      publicSummary: `PR template ${template.path} with ${template.headings.length} headings and ${template.checklistItems} checklist items.`,
      recommendation: "Preserve template requirements before using advisory recent PR style."
    }));
  }
}

function repoConstitutionFindingEvidence(finding: RepoConstitutionFinding, index: number): FoundryEvidenceItem {
  return safeItem({
    kind: "finding",
    sourceType: "repo_constitution",
    sourceRef: { sourceType: "repo_constitution", sourceId: "constitution-findings", index },
    confidence: finding.severity === "warning" ? "medium" : "low",
    severity: finding.severity,
    code: `repo_constitution:${finding.code}`,
    publicSummary: finding.message,
    recommendation: finding.recommendation
  });
}

function isRepoConstitutionEvidenceInput(value: unknown): value is RepoConstitution {
  if (!isRecord(value) || value.schemaVersion !== 1 || !Array.isArray(value.findings)) return false;
  const pullRequests = value.pullRequests;
  return value.findings.every(isRepoConstitutionFinding) &&
    isRecord(pullRequests) &&
    Array.isArray(pullRequests.templates) &&
    pullRequests.templates.every(isPullRequestTemplateSummary);
}

function isRepoConstitutionFinding(value: unknown): value is RepoConstitutionFinding {
  return isRecord(value) &&
    typeof value.code === "string" &&
    (value.severity === "info" || value.severity === "warning") &&
    typeof value.message === "string" &&
    typeof value.recommendation === "string";
}

function isPullRequestTemplateSummary(value: unknown): boolean {
  return isRecord(value) &&
    typeof value.path === "string" &&
    typeof value.contentProvided === "boolean" &&
    Array.isArray(value.headings) &&
    value.headings.every((heading) => typeof heading === "string") &&
    typeof value.checklistItems === "number";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function safeItem(input: {
  kind: FoundryEvidenceKind;
  sourceType: FoundryEvidenceItem["sourceType"];
  sourceRef: FoundryEvidenceSourceRef;
  confidence: FoundryEvidenceItem["confidence"];
  severity?: FoundryEvidenceItem["severity"];
  code?: string;
  path?: string;
  publicSummary: string;
  recommendation?: string;
  sensitive?: boolean;
  rawPayloadOmitted?: boolean;
  suppressionCandidate?: FoundrySuppressionCandidate;
}): FoundryEvidenceItem {
  const summary = publicSafeSummary(input.publicSummary);
  const recommendation = input.recommendation ? publicSafeSummary(input.recommendation) : { value: undefined, redacted: false };
  const code = input.code ? publicSafeText(input.code) : { value: undefined, redacted: false };
  const path = input.path ? publicSafeText(input.path) : { value: undefined, redacted: false };
  const sourceId = publicSafeText(input.sourceRef.sourceId);
  const sourcePath = input.sourceRef.path ? publicSafeText(input.sourceRef.path) : { value: undefined, redacted: false };
  const redacted = summary.redacted || recommendation.redacted || code.redacted || path.redacted || sourceId.redacted || sourcePath.redacted;
  return {
    id: "",
    kind: input.kind,
    sourceType: input.sourceType,
    sourceRef: {
      ...input.sourceRef,
      sourceId: sourceId.value,
      path: sourcePath.value
    },
    confidence: input.confidence,
    severity: input.severity,
    code: code.value,
    path: path.value,
    publicSummary: summary.value,
    recommendation: recommendation.value,
    publicSafetyClass: publicSafetyClass(input.sensitive, redacted, input.rawPayloadOmitted),
    redactionStatus: redactionStatus(redacted, input.rawPayloadOmitted),
    suppressionCandidate: input.suppressionCandidate
  };
}

function publicSafetyClass(sensitive?: boolean, redacted?: boolean, rawPayloadOmitted?: boolean): FoundryPublicSafetyClass {
  if (sensitive) return "sensitive";
  if (redacted || rawPayloadOmitted) return "redacted";
  return "public";
}

function redactionStatus(redacted?: boolean, rawPayloadOmitted?: boolean): FoundryRedactionStatus {
  if (redacted && rawPayloadOmitted) return "redacted_and_omitted_raw_payload";
  if (redacted) return "redacted";
  if (rawPayloadOmitted) return "omitted_raw_payload";
  return "none";
}

function suppressionCandidateForPath(path: string | undefined, code: string): FoundrySuppressionCandidate | undefined {
  if (!path) return code === "ARCH005_MISSING_DIRECTORY" || code === "ARCH024_AGENT_HARNESS" ? prerequisite("repo_profile_mismatch") : undefined;
  const normalized = path.replace(/\\/g, "/");
  if (/node_modules|vendor\/|third_party\//i.test(normalized)) return prerequisite("vendored_code");
  if (/generated|dist\/|build\/|coverage\/|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|bun\.lockb?/i.test(normalized)) return prerequisite("generated_file");
  if (/__fixtures__|fixtures?\/|testdata\/|\.fixture\./i.test(normalized)) return prerequisite("fixture_or_test_data");
  if (/^(docs|examples)\//i.test(normalized) || /\.(mdx?|rst|adoc)$/i.test(normalized)) return prerequisite("docs_example");
  if (/^(?:src\/)?(?:index|main|server|app|route)\.(?:ts|tsx|js|jsx|mjs|cjs|py|go|rs)$/i.test(normalized)) return prerequisite("conventional_entrypoint");
  return undefined;
}

function prerequisite(category: FoundrySuppressionCategory): FoundrySuppressionCandidate {
  const match = foundrySuppressionPrerequisites().find((item) => item.category === category);
  return {
    category,
    reason: match?.reason ?? "Review this suppression category before routing.",
    prerequisiteIssue: match?.issue ?? "https://github.com/tonycdr-prog/architect-mcp/issues/324"
  };
}

function summarizeItems(items: FoundryEvidenceItem[]): FoundryEvidenceInventory["summary"] {
  return {
    totalEvidence: items.length,
    bySourceType: countBy(items.map((item) => item.sourceType)),
    byConfidence: countBy(items.map((item) => item.confidence)),
    byPublicSafetyClass: countBy(items.map((item) => item.publicSafetyClass)),
    redacted: items.filter((item) => item.redactionStatus === "redacted" || item.redactionStatus === "redacted_and_omitted_raw_payload").length,
    omittedRawPayloads: items.filter((item) => item.redactionStatus === "omitted_raw_payload" || item.redactionStatus === "redacted_and_omitted_raw_payload").length,
    suppressionCandidates: items.filter((item) => item.suppressionCandidate).length,
    coverageCaveats: items.filter((item) => item.kind === "coverage_caveat").length
  };
}

function countBy(values: string[]): Record<string, number> {
  return values.reduce<Record<string, number>>((acc, value) => {
    acc[value] = (acc[value] ?? 0) + 1;
    return acc;
  }, {});
}
