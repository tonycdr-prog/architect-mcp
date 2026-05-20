import type { FoundryActionabilityReport } from "./foundryActionabilityTypes.js";
import type { FoundryDecisionLedgerReport, FoundryDecisionRoute } from "./foundryDecisionLedgerTypes.js";
import type { FoundryEvalCorpusCase, FoundryEvalRegressionAssertion } from "./foundryEvalCorpusTypes.js";
import type { FoundryEvidenceInventory, FoundrySuppressionCategory } from "./foundryEvidenceTypes.js";
import type { FoundryForgePreviewKind, FoundryForgePreviewReport } from "./foundryForgeTypes.js";
import type { RepoConstitution } from "./repoConstitutionTypes.js";

export type FoundryEvalRegressionContext = {
  item: FoundryEvalCorpusCase;
  constitution: RepoConstitution;
  inventory: FoundryEvidenceInventory;
  actionability: FoundryActionabilityReport;
  ledger: FoundryDecisionLedgerReport;
  forge?: FoundryForgePreviewReport;
  suppressionCategories: FoundrySuppressionCategory[];
};

export function evaluateFoundryEvalRegressionAssertions(
  context: FoundryEvalRegressionContext
): FoundryEvalRegressionAssertion[] {
  return [
    assertion(310, hasSuppression(context, "docs_example") && hasRoute(context, "exception"), "Docs examples route to suppressible exception candidates instead of PR previews."),
    assertion(311, context.inventory.coverage.scanTruncated === true && hasCoverageCaveat(context), "Large-repo scan truncation remains visible as coverage evidence."),
    assertion(312, hasEvidenceCode(context, "TSX_ENV_CONTEXT") && hasRoute(context, "no_op"), "TSX/env-like context evidence routes away from PR mutation."),
    assertion(313, hasSuppression(context, "generated_file") && hasRoute(context, "exception"), "Generated/data-file oversized noise routes as a suppression exception."),
    assertion(314, hasPrTemplateAndRecentStyle(context) && hasPreviewKind(context, "pull_request"), "Repo PR template signals and recent accepted PR style feed PR-preview shape."),
    assertion(315, hasSuppression(context, "conventional_entrypoint") && hasRoute(context, "exception"), "Conventional entrypoint noise routes as an exception candidate."),
    assertion(316, hasEnvEvidence(context) && hasAnyRoute(context), "Environment-related evidence survives normalization and receives a route."),
    assertion(317, hasSuppression(context, "docs_example") && hasDocsEvidence(context), "Documentation intelligence distinguishes docs/example findings from source findings."),
    assertion(318, hasConstitutionFinding(context, "MISSING_AGENT_INSTRUCTIONS") && hasRoute(context, "architect_issue"), "Missing AGENTS.md in external-style repos routes to an issue preview, not a hard gate."),
    assertion(319, hasLanguageProfileContext(context) && hasAnyRoute(context), "Non-Node language/framework profile evidence influences routing context.")
  ];
}

function assertion(issue: number, passed: boolean, evidence: string): FoundryEvalRegressionAssertion {
  return { issue, passed, evidence };
}

function hasSuppression(context: FoundryEvalRegressionContext, category: FoundrySuppressionCategory): boolean {
  return context.suppressionCategories.includes(category);
}

function hasRoute(context: FoundryEvalRegressionContext, route: FoundryDecisionRoute): boolean {
  return (context.ledger.summary.byRoute[route] ?? 0) > 0;
}

function hasAnyRoute(context: FoundryEvalRegressionContext): boolean {
  return Object.values(context.ledger.summary.byRoute).some((count) => count > 0);
}

function hasPreviewKind(context: FoundryEvalRegressionContext, kind: FoundryForgePreviewKind): boolean {
  return context.forge?.previews.some((preview) => preview.kind === kind) ?? false;
}

function hasCoverageCaveat(context: FoundryEvalRegressionContext): boolean {
  return context.inventory.coverage.caveats.length > 0 ||
    context.inventory.evidence.some((entry) => entry.kind === "coverage_caveat");
}

function hasEvidenceCode(context: FoundryEvalRegressionContext, code: string): boolean {
  return context.inventory.evidence.some((entry) => entry.code === code);
}

function hasDocsEvidence(context: FoundryEvalRegressionContext): boolean {
  return context.inventory.evidence.some((entry) =>
    /docs|example|\.mdx?$/i.test(`${entry.path ?? ""} ${entry.publicSummary}`)
  );
}

function hasPrTemplateAndRecentStyle(context: FoundryEvalRegressionContext): boolean {
  return context.constitution.pullRequests.templates.length > 0 &&
    context.constitution.pullRequests.recentStyle.acceptedSamples > 0;
}

function hasEnvEvidence(context: FoundryEvalRegressionContext): boolean {
  return context.inventory.evidence.some((entry) =>
    /ENV|env|environment/i.test(`${entry.code ?? ""} ${entry.path ?? ""} ${entry.publicSummary}`)
  );
}

function hasConstitutionFinding(context: FoundryEvalRegressionContext, code: "MISSING_AGENT_INSTRUCTIONS"): boolean {
  return context.constitution.findings.some((finding) => finding.code === code);
}

function hasLanguageProfileContext(context: FoundryEvalRegressionContext): boolean {
  return context.item.repo.type === "python_framework" ||
    context.item.repo.type === "rust_cli" ||
    context.constitution.summary.primaryLanguages.includes("Python") ||
    context.constitution.summary.primaryLanguages.includes("Rust");
}
