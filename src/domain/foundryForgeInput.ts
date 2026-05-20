import type { FoundryDecisionLedgerEntry, FoundryDecisionLedgerReport, FoundryDecisionRoute } from "./foundryDecisionLedgerTypes.js";
import type { RepoConstitution } from "./repoConstitutionTypes.js";

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

export function isForgeRepoConstitution(value: unknown): value is RepoConstitution {
  if (!isRecord(value) || value.schemaVersion !== 1 || !isRecord(value.pullRequests)) return false;
  const pullRequests = value.pullRequests;
  const ci = value.ci;
  return Array.isArray(pullRequests.templates) &&
    pullRequests.templates.every(isTemplateSummary) &&
    isRecentStyleSummary(pullRequests.recentStyle) &&
    Array.isArray(value.findings) &&
    isRecord(ci) &&
    Array.isArray(ci.workflows);
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

function isTemplateSummary(value: unknown): boolean {
  return isRecord(value) &&
    typeof value.path === "string" &&
    typeof value.contentProvided === "boolean" &&
    typeof value.hiddenCommentOnly === "boolean" &&
    Array.isArray(value.headings) &&
    value.headings.every((heading) => typeof heading === "string") &&
    typeof value.checklistItems === "number" &&
    typeof value.mentionsLinkedIssues === "boolean" &&
    typeof value.mentionsReleaseNotes === "boolean" &&
    typeof value.mentionsVerification === "boolean";
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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
