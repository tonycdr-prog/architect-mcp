import type { FoundryDecisionLedgerEntry, FoundryDecisionLedgerReport, FoundryDecisionRoute } from "./foundryDecisionLedgerTypes.js";
import type {
  PullRequestTemplateSummary,
  RecentPullRequestStyleSummary,
  RepoConstitution,
  RepoConstitutionFinding,
  WorkflowSummary
} from "./repoConstitutionTypes.js";

const ROUTES = new Set<FoundryDecisionRoute>([
  "pr_preview",
  "architect_issue",
  "exception",
  "no_op",
  "ask_human"
]);

export function isFoundryDecisionLedgerReport(value: unknown): value is FoundryDecisionLedgerReport {
  if (!isRecord(value) || value.schemaVersion !== 1 || !Array.isArray(value.entries)) return false;
  return value.entries.every(isFoundryDecisionLedgerEntry);
}

export function normalizeForgeRepoConstitution(value: unknown): RepoConstitution | undefined {
  if (!isRecord(value) || value.schemaVersion !== 1 || !isRecord(value.pullRequests)) return undefined;
  const pullRequests = value.pullRequests;
  if (!Array.isArray(pullRequests.templates) ||
    !isRecentStyleSummary(pullRequests.recentStyle) ||
    !Array.isArray(value.findings) ||
    !value.findings.every(isRepoConstitutionFinding)) {
    return undefined;
  }

  const templates = pullRequests.templates.map(normalizeTemplateSummary);
  if (templates.some((template) => template === undefined)) return undefined;
  const ci = normalizeCi(value.ci);

  return {
    ...(value as RepoConstitution),
    schemaVersion: 1,
    pullRequests: {
      ...(pullRequests as RepoConstitution["pullRequests"]),
      templates: templates as PullRequestTemplateSummary[],
      recentStyle: pullRequests.recentStyle as RecentPullRequestStyleSummary,
      precedence: stringArrayOrEmpty(pullRequests.precedence)
    },
    ci,
    findings: value.findings as RepoConstitutionFinding[]
  };
}

function isFoundryDecisionLedgerEntry(value: unknown): value is FoundryDecisionLedgerEntry {
  return isRecord(value) &&
    typeof value.id === "string" &&
    ROUTES.has(value.route as FoundryDecisionRoute) &&
    Array.isArray(value.evidenceIds) &&
    value.evidenceIds.every((id) => typeof id === "string") &&
    isRecord(value.source) &&
    typeof value.source.score === "number" &&
    Number.isFinite(value.source.score) &&
    typeof value.decisionReason === "string" &&
    Array.isArray(value.verificationRequirements) &&
    value.verificationRequirements.every((item) => typeof item === "string") &&
    (value.approvalState === "approval_required" || value.approvalState === "human_required" || value.approvalState === "not_required") &&
    isRecord(value.mutation) &&
    value.mutation.serverMutationAllowed === false;
}

function normalizeTemplateSummary(value: unknown): PullRequestTemplateSummary | undefined {
  if (!isRecord(value) ||
    typeof value.path !== "string" ||
    typeof value.contentProvided !== "boolean" ||
    !optionalBoolean(value.hiddenCommentOnly) ||
    !Array.isArray(value.headings) ||
    !value.headings.every((heading) => typeof heading === "string") ||
    typeof value.checklistItems !== "number" ||
    !optionalBoolean(value.mentionsLinkedIssues) ||
    !optionalBoolean(value.mentionsReleaseNotes) ||
    !optionalBoolean(value.mentionsVerification)) {
    return undefined;
  }
  return {
    ...(value as PullRequestTemplateSummary),
    path: value.path,
    contentProvided: value.contentProvided,
    hiddenCommentOnly: value.hiddenCommentOnly === true,
    headings: value.headings,
    checklistItems: value.checklistItems,
    mentionsLinkedIssues: value.mentionsLinkedIssues === true,
    mentionsReleaseNotes: value.mentionsReleaseNotes === true,
    mentionsVerification: value.mentionsVerification === true
  };
}

function isRecentStyleSummary(value: unknown): boolean {
  return isRecord(value) &&
    value.advisory === true &&
    Array.isArray(value.commonHeadings) &&
    value.commonHeadings.every((item) =>
      isRecord(item) &&
      typeof item.heading === "string" &&
      typeof item.count === "number"
    );
}

function isRepoConstitutionFinding(value: unknown): value is RepoConstitutionFinding {
  return isRecord(value) &&
    typeof value.code === "string" &&
    (value.severity === "info" || value.severity === "warning") &&
    typeof value.message === "string" &&
    typeof value.recommendation === "string";
}

function normalizeCi(value: unknown): RepoConstitution["ci"] {
  const ci = isRecord(value) ? value : {};
  const workflows = Array.isArray(ci.workflows)
    ? ci.workflows.filter(isRecord) as WorkflowSummary[]
    : [];
  return {
    ...(ci as RepoConstitution["ci"]),
    workflows,
    labelerConfigPaths: stringArrayOrEmpty(ci.labelerConfigPaths)
  };
}

function stringArrayOrEmpty(value: unknown): string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string") ? value : [];
}

function optionalBoolean(value: unknown): boolean {
  return value === undefined || typeof value === "boolean";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
