import { scoreFoundryActionability } from "./foundryActionability.js";
import { routeFoundryDecisions } from "./foundryDecisionLedger.js";
import type { FoundryDecisionRoute } from "./foundryDecisionLedgerTypes.js";
import { evaluateFoundryEvalRegressionAssertions } from "./foundryEvalCorpusAssertions.js";
import { foundryEvalCorpusFixtureCases } from "./foundryEvalCorpusFixtures.js";
import type {
  FoundryEvalCorpusCase,
  FoundryEvalCorpusInput,
  FoundryEvalCorpusReport,
  FoundryEvalCorpusResult,
  FoundryEvalPublicSafety
} from "./foundryEvalCorpusTypes.js";
import { normalizeFoundryEvidence } from "./foundryEvidence.js";
import type { FoundryEvidenceInventoryInput, FoundrySuppressionCategory } from "./foundryEvidenceTypes.js";
import { forgeFoundryPreviews } from "./foundryForge.js";
import type { FoundryForgePreviewKind } from "./foundryForgeTypes.js";
import { deriveRepoConstitution } from "./repoConstitution.js";
import type { ReviewReport } from "./types.js";

export type {
  FoundryEvalCorpusCase,
  FoundryEvalCorpusInput,
  FoundryEvalCorpusReport,
  FoundryEvalCorpusRepoSize,
  FoundryEvalCorpusRepoType,
  FoundryEvalCorpusResult,
  FoundryEvalPublicSafety
} from "./foundryEvalCorpusTypes.js";

const REQUIRED_ROUTES: FoundryDecisionRoute[] = [
  "pr_preview",
  "architect_issue",
  "exception",
  "no_op",
  "ask_human"
];

const REQUIRED_REGRESSION_ISSUES = [310, 311, 312, 313, 314, 315, 316, 317, 318, 319];

export function foundryEvalCorpusCases(): FoundryEvalCorpusCase[] {
  return foundryEvalCorpusFixtureCases.map((item) => ({
    ...item,
    repo: { ...item.repo },
    expectedRoutes: [...item.expectedRoutes],
    expectedNoisePatterns: [...item.expectedNoisePatterns],
    regressionIssues: [...item.regressionIssues],
    files: item.files?.map((file) => ({ ...file })),
    artifacts: item.artifacts?.map((artifact) => ({ ...artifact })),
    recentPullRequests: item.recentPullRequests?.map((pullRequest) => ({ ...pullRequest })),
    evidence: cloneEvidenceInput(item.evidence),
    verificationHints: item.verificationHints ? [...item.verificationHints] : undefined
  }));
}

export function runFoundryEvalCorpus(input: FoundryEvalCorpusInput = {}): FoundryEvalCorpusReport {
  const cases = foundryEvalCorpusCases();
  const caseIds = new Set(input.caseIds ?? []);
  const knownCaseIds = new Set(cases.map((item) => item.id));
  const unknownCaseIds = [...caseIds].filter((id) => !knownCaseIds.has(id)).sort();
  const selectedSubset = caseIds.size > 0;
  const selectedCases = cases.filter((item) => caseIds.size === 0 || caseIds.has(item.id));
  const results = selectedCases.map((item) => evaluateCase(item, input.includePreviews ?? true));
  const byRoute = mergeCounts(results.map((result) => result.routes));
  const previewKinds = mergeCounts(results.map((result) => result.previewKinds));
  const regressionIssuesCovered = uniqueNumbers(results.flatMap((result) => result.regressionIssues));
  const routeCoverage = Object.fromEntries(REQUIRED_ROUTES.map((route) => [route, (byRoute[route] ?? 0) > 0])) as Record<FoundryDecisionRoute, boolean>;
  const regressionIssues = Object.fromEntries(REQUIRED_REGRESSION_ISSUES.map((issue) => [String(issue), regressionIssuesCovered.includes(issue)]));
  const publicSafety = summarizePublicSafety(results);
  const noMutation = results.every((result) => result.noMutation.passed);
  const failed = results.filter((result) => result.status === "fail").length;
  const corpusPassed = failed === 0 &&
    selectedCases.length > 0 &&
    unknownCaseIds.length === 0 &&
    Object.values(routeCoverage).every(Boolean) &&
    Object.values(regressionIssues).every(Boolean) &&
    publicSafety.passed &&
    noMutation;
  const status = corpusPassed ? "pass" : selectedSubset &&
    failed === 0 &&
    selectedCases.length > 0 &&
    unknownCaseIds.length === 0 &&
    publicSafety.passed &&
    noMutation ? "partial" : "fail";

  return {
    schemaVersion: 1,
    status,
    summary: {
      totalCases: results.length,
      passed: results.length - failed,
      failed,
      byRepoSize: countBy(results.map((result) => result.repo.size)),
      byRoute,
      previewKinds,
      suppressionCategories: uniqueFlat(results.map((result) => result.suppressionCategories)),
      regressionIssuesCovered,
      selectedSubset,
      unknownCaseIds,
      offlineNetworkRequired: false,
      liveSmokeAvailable: true,
      serverWritesPerformed: 0
    },
    requirements: {
      offlineFixtures: selectedCases.length > 0,
      optionalLiveSmoke: true,
      routeCoverage,
      publicSafety: publicSafety.passed,
      noMutation,
      regressionIssues
    },
    manifest: {
      cases: selectedCases.map((item) => ({
        id: item.id,
        repo: item.repo,
        expectedRoutes: item.expectedRoutes,
        expectedNoisePatterns: item.expectedNoisePatterns,
        regressionIssues: item.regressionIssues
      }))
    },
    results,
    liveSmoke: liveSmokeFor(selectedCases),
    publicSafety
  };
}

function evaluateCase(item: FoundryEvalCorpusCase, includePreviews: boolean): FoundryEvalCorpusResult {
  const constitution = deriveRepoConstitution({
    files: item.files,
    artifacts: item.artifacts,
    recentPullRequests: item.recentPullRequests
  });
  const inventory = normalizeFoundryEvidence({ ...item.evidence, repoConstitution: constitution });
  const actionability = scoreFoundryActionability({ inventory, verificationHints: item.verificationHints });
  const ledger = routeFoundryDecisions({ actionability });
  const forge = includePreviews ? forgeFoundryPreviews({ ledger, repoConstitution: constitution }) : undefined;
  const suppressionCategories = uniqueFlat(inventory.evidence.map((entry) => entry.suppressionCandidate?.category ? [entry.suppressionCandidate.category] : []));
  const previewKinds = forge ? previewKindCounts(forge.previews.map((preview) => preview.kind)) : {};
  const missingExpectedRoutes = item.expectedRoutes.filter((route) => (ledger.summary.byRoute[route] ?? 0) === 0);
  const regressionAssertions = evaluateFoundryEvalRegressionAssertions({
    item,
    constitution,
    inventory,
    actionability,
    ledger,
    forge,
    suppressionCategories
  });
  const regressionIssues = regressionAssertions.filter((assertion) => assertion.passed).map((assertion) => assertion.issue);
  const missingDeclaredRegressionIssues = item.regressionIssues.filter((issue) => !regressionIssues.includes(issue));
  const publicSafety = evaluatePublicSafety({
    inventory: {
      publicSafety: inventory.publicSafety,
      evidence: inventory.evidence
    },
    actionability: {
      publicSafety: actionability.publicSafety,
      assessments: actionability.assessments
    },
    ledger: {
      publicSafety: ledger.publicSafety,
      entries: ledger.entries
    },
    forge: forge ? {
      publicSafety: forge.publicSafety,
      previews: forge.previews
    } : undefined,
    routes: ledger.summary.byRoute,
    previewKinds,
    suppressionCategories
  });
  const noMutation = ledger.summary.serverWritesPerformed === 0 &&
    (!forge || forge.summary.serverWritesPerformed === 0) &&
    inventory.publicSafety.mutationAllowed === false &&
    actionability.publicSafety.mutationAllowed === false &&
    ledger.publicSafety.mutationAllowed === false &&
    (!forge || forge.publicSafety.mutationAllowed === false);

  return {
    id: item.id,
    repo: item.repo,
    status: missingExpectedRoutes.length === 0 && missingDeclaredRegressionIssues.length === 0 && publicSafety.passed && noMutation ? "pass" : "fail",
    expectedRoutes: item.expectedRoutes,
    missingExpectedRoutes,
    routes: ledger.summary.byRoute,
    previewKinds,
    suppressionCategories,
    regressionIssues,
    declaredRegressionIssues: item.regressionIssues,
    missingDeclaredRegressionIssues,
    regressionAssertions,
    expectedNoisePatterns: item.expectedNoisePatterns,
    publicSafety,
    noMutation: { passed: noMutation, serverWritesPerformed: 0, mutationAllowed: false }
  };
}

function liveSmokeFor(selectedCases: FoundryEvalCorpusCase[]): FoundryEvalCorpusReport["liveSmoke"] {
  return {
    mode: "optional_read_only",
    networkRequired: true,
    command: "architect-mcp-tui foundry-audit --repo-path <fresh-public-checkout> --public-summary",
    selectedPublicRepos: selectedCases.map((item) => item.repo.slug),
    noMutationChecks: [
      "Use a fresh checkout and record `git status --short` before and after the audit.",
      "Require Foundry public summaries to report zero server writes.",
      "Do not create branches, commits, issues, pull requests, comments, labels, files, or .architect-mcp state."
    ]
  };
}

function evaluatePublicSafety(summary: unknown): FoundryEvalPublicSafety {
  const leaks = publicSafetyLeaks(JSON.stringify(summary));
  return {
    passed: leaks.length === 0,
    leaks,
    rawPayloadsIncluded: false,
    rawRepoContentIncluded: false,
    localPathsIncluded: false,
    tokenValuesIncluded: false
  };
}

function summarizePublicSafety(results: FoundryEvalCorpusResult[]): FoundryEvalPublicSafety {
  const leaks = uniqueFlat(results.map((result) => result.publicSafety.leaks));
  return {
    passed: leaks.length === 0 && results.every((result) => result.publicSafety.passed),
    leaks,
    rawPayloadsIncluded: false,
    rawRepoContentIncluded: false,
    localPathsIncluded: false,
    tokenValuesIncluded: false
  };
}

function publicSafetyLeaks(text: string): string[] {
  const leaks: string[] = [];
  if (/(?:\/Users|\/home|\/private\/tmp|\/tmp|\/var\/folders|\/Volumes)\/|[A-Za-z]:\\/i.test(text)) leaks.push("local_path");
  if (/\b(?:ghp_|sk-|xoxb-|AKIA)[A-Za-z0-9_-]{8,}\b/i.test(text)) leaks.push("token_value");
  if (/RAW_PRIVATE_PAYLOAD|private stack trace|secret internal path/i.test(text)) leaks.push("raw_payload");
  return leaks;
}

function previewKindCounts(values: FoundryForgePreviewKind[]): Record<string, number> {
  return countBy(values);
}

function mergeCounts(counts: Array<Record<string, number>>): Record<string, number> {
  const merged: Record<string, number> = {};
  for (const count of counts) {
    for (const [key, value] of Object.entries(count)) {
      merged[key] = (merged[key] ?? 0) + value;
    }
  }
  return merged;
}

function countBy(values: string[]): Record<string, number> {
  return values.reduce<Record<string, number>>((acc, value) => {
    acc[value] = (acc[value] ?? 0) + 1;
    return acc;
  }, {});
}

function uniqueFlat<T extends string>(values: T[][]): T[] {
  return [...new Set(values.flat())].sort();
}

function uniqueNumbers(values: number[]): number[] {
  return [...new Set(values)].sort((a, b) => a - b);
}

function cloneEvidenceInput(input: Omit<FoundryEvidenceInventoryInput, "repoConstitution">): Omit<FoundryEvidenceInventoryInput, "repoConstitution"> {
  return {
    findings: input.findings?.map((item) => ({ ...item })),
    reviewReports: input.reviewReports?.map(cloneReviewReport),
    externalFindings: input.externalFindings?.map((item) => ({ ...item })),
    verification: input.verification?.map((item) => ({ ...item }))
  };
}

function cloneReviewReport(report: ReviewReport): ReviewReport {
  return {
    ...report,
    summary: { ...report.summary, coverageCaveats: [...report.summary.coverageCaveats] },
    coverage: {
      ...report.coverage,
      topScannedDirectories: report.coverage.topScannedDirectories.map((item) => ({ ...item })),
      findingHistogram: report.coverage.findingHistogram.map((item) => ({ ...item })),
      caveats: [...report.coverage.caveats]
    },
    groups: report.groups.map((item) => ({ ...item, samplePaths: [...item.samplePaths] })),
    priorityFindings: report.priorityFindings.map((item) => ({ ...item })),
    violations: report.violations.map((item) => ({ ...item }))
  };
}
