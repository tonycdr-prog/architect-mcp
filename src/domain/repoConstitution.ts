import { normalizePath } from "./pathRules.js";
import {
  addPathProvenance,
  collectExistingPaths,
  collectPackageMetadata,
  collectPullRequestTemplates,
  collectWorkflows,
  inferRepoShape,
  isPullRequestTemplatePath,
  normalizeHeading,
  summarizeLanguages,
  summarizeRecentPullRequestStyle
} from "./repoConstitutionSignals.js";
import {
  AGENT_INSTRUCTION_PATHS,
  HUMAN_INSTRUCTION_PATHS,
  RELEASE_DOC_PATTERNS,
  SECURITY_POLICY_PATHS,
  type PackageMetadataSummary,
  type PullRequestTemplateSummary,
  type RecentPullRequestStyleSummary,
  type RepoConstitution,
  type RepoConstitutionFinding,
  type RepoConstitutionInput,
  type RepoConstitutionProvenance,
  type WorkflowSummary
} from "./repoConstitutionTypes.js";

export type {
  PackageMetadataSummary,
  PullRequestTemplateSummary,
  RecentPullRequestStyleSummary,
  RepoConstitution,
  RepoConstitutionArtifact,
  RepoConstitutionFinding,
  RepoConstitutionInput,
  RepoConstitutionProvenance,
  RepoConstitutionPullRequest,
  WorkflowSummary
} from "./repoConstitutionTypes.js";

export function deriveRepoConstitution(input: RepoConstitutionInput): RepoConstitution {
  const files = (input.files ?? []).map((file) => ({ ...file, path: normalizePath(file.path) }));
  const artifacts = new Map((input.artifacts ?? []).map((artifact) => [normalizePath(artifact.path), artifact.content]));
  const filePaths = new Set([...files.map((file) => file.path), ...artifacts.keys()]);
  const provenance: RepoConstitutionProvenance[] = [];

  const agentInstructionPaths = collectExistingPaths(filePaths, AGENT_INSTRUCTION_PATHS);
  const humanInstructionPaths = collectExistingPaths(filePaths, HUMAN_INSTRUCTION_PATHS);
  const securityPolicyPaths = collectExistingPaths(filePaths, SECURITY_POLICY_PATHS);
  addPathProvenance(provenance, "repo-instructions", agentInstructionPaths, "Agent instruction file.");
  addPathProvenance(provenance, "repo-instructions", humanInstructionPaths, "Human contributor or setup guidance.");
  addPathProvenance(provenance, "repo-instructions", securityPolicyPaths, "Security reporting policy.");

  const templates = collectPullRequestTemplates(filePaths, artifacts, provenance);
  const recentStyle = summarizeRecentPullRequestStyle(input.recentPullRequests ?? [], provenance);
  const workflows = collectWorkflows(filePaths, artifacts, provenance);
  const labelerConfigPaths = [...filePaths].filter((path) => /^\.github\/(labeler|labels)\.ya?ml$/i.test(path)).sort();
  addPathProvenance(provenance, "labels", labelerConfigPaths, "Label configuration.");

  const changelogPaths = [...filePaths].filter((path) => /(^|\/)CHANGELOG\.md$/i.test(path)).sort();
  const releaseWorkflowPaths = workflows.filter((workflow) => /release|publish|deploy/i.test(`${workflow.path} ${workflow.name ?? ""}`)).map((workflow) => workflow.path);
  const releaseDocPaths = [...filePaths].filter((path) => RELEASE_DOC_PATTERNS.some((pattern) => pattern.test(path))).sort();
  addPathProvenance(provenance, "release-policy", changelogPaths, "Changelog file.");
  addPathProvenance(provenance, "release-policy", releaseWorkflowPaths, "Release or publish workflow.");
  addPathProvenance(provenance, "release-policy", releaseDocPaths, "Release documentation.");

  const packageMetadata = collectPackageMetadata(filePaths, artifacts, provenance);
  const primaryLanguages = summarizeLanguages(filePaths);
  const packageManagers = [...new Set(packageMetadata.map((metadata) => metadata.manager))].sort();
  const findings = collectFindings({ templates, recentStyle, workflows, agentInstructionPaths });

  return {
    schemaVersion: 1,
    summary: {
      hardSignals: agentInstructionPaths.length + humanInstructionPaths.length + templates.length + workflows.length + packageMetadata.length,
      advisorySignals: recentStyle.acceptedSamples,
      warnings: findings.filter((finding) => finding.severity === "warning").length,
      missingRecommendedSignals: missingSignals({ agentInstructionPaths, humanInstructionPaths, templates, workflows, packageMetadata }),
      primaryLanguages,
      packageManagers,
      repoShape: inferRepoShape({ filePaths, packageMetadata, primaryLanguages })
    },
    instructions: { agentInstructionPaths, humanInstructionPaths, securityPolicyPaths },
    pullRequests: {
      templates,
      recentStyle,
      precedence: [
        "Explicit repository instructions and security policy outrank all inferred signals.",
        "Repository PR templates outrank recent PR body style.",
        "Recent accepted merged PR style is advisory fallback evidence only.",
        "Bot-only PR bodies are ignored unless a future dependency-only mode explicitly opts in."
      ]
    },
    ci: { workflows, labelerConfigPaths },
    release: { changelogPaths, releaseWorkflowPaths, releaseDocPaths },
    packageMetadata,
    maintainerConstraints: summarizeMaintainerConstraints({
      agentInstructionPaths,
      humanInstructionPaths,
      templates,
      workflows,
      securityPolicyPaths,
      releaseWorkflowPaths,
      releaseDocPaths
    }),
    findings,
    provenance,
    publicSafety: {
      rawContentIncluded: false,
      mutationAllowed: false
    }
  };
}

export function isRepoConstitutionArtifactPath(path: string): boolean {
  const normalized = normalizePath(path);
  return AGENT_INSTRUCTION_PATHS.includes(normalized) ||
    HUMAN_INSTRUCTION_PATHS.includes(normalized) ||
    SECURITY_POLICY_PATHS.includes(normalized) ||
    isPullRequestTemplatePath(normalized) ||
    /^\.github\/workflows\/[^/]+\.ya?ml$/i.test(normalized) ||
    /^\.github\/(labeler|labels)\.ya?ml$/i.test(normalized) ||
    /^package\.json$/i.test(normalized) ||
    /^Cargo\.toml$/i.test(normalized) ||
    /^pyproject\.toml$/i.test(normalized) ||
    /^go\.mod$/i.test(normalized) ||
    /(^|\/)CHANGELOG\.md$/i.test(normalized) ||
    RELEASE_DOC_PATTERNS.some((pattern) => pattern.test(normalized));
}

function collectFindings(input: {
  templates: PullRequestTemplateSummary[];
  recentStyle: RecentPullRequestStyleSummary;
  workflows: WorkflowSummary[];
  agentInstructionPaths: string[];
}): RepoConstitutionFinding[] {
  const findings: RepoConstitutionFinding[] = [];
  if (input.templates.length === 0) {
    findings.push({
      code: "MISSING_PR_TEMPLATE",
      severity: "info",
      message: "No repository pull request template was found.",
      recommendation: "Use recent accepted PR style as advisory fallback and ask a maintainer before mutating external repos."
    });
  }
  if (input.templates.some((template) => template.contentProvided && (template.hiddenCommentOnly || (template.headings.length === 0 && template.checklistItems === 0)))) {
    findings.push({
      code: "HIDDEN_OR_SPARSE_PR_TEMPLATE",
      severity: "warning",
      message: "A pull request template is hidden-comment-only or too sparse to define body structure.",
      recommendation: "Preserve hidden template requirements, then use recent accepted PR bodies as advisory style evidence."
    });
  }
  if (input.templates.length > 0 && input.recentStyle.commonHeadings.length > 0 && templateHeadingsDiverge(input.templates, input.recentStyle)) {
    findings.push({
      code: "RECENT_PR_STYLE_DIVERGES",
      severity: "warning",
      message: "The PR template may be stale or not representative because recent accepted PR headings do not overlap the template headings.",
      recommendation: "Treat the template as authoritative for required sections, and use recent accepted PR style as advisory formatting evidence before asking a maintainer."
    });
  }
  if (input.workflows.length === 0) {
    findings.push({
      code: "MISSING_CI",
      severity: "warning",
      message: "No GitHub Actions workflows were found.",
      recommendation: "Ask for the repository verification command before generating PR or issue previews."
    });
  }
  if (input.agentInstructionPaths.length === 0) {
    findings.push({
      code: "MISSING_AGENT_INSTRUCTIONS",
      severity: "info",
      message: "No repo-local agent instructions were found.",
      recommendation: "Use human-facing docs and repository templates as hard constraints; do not invent agent-specific rules."
    });
  }
  return findings;
}

function templateHeadingsDiverge(templates: PullRequestTemplateSummary[], recentStyle: RecentPullRequestStyleSummary): boolean {
  const templateHeadings = new Set(templates.flatMap((template) => template.headings.map(normalizeHeading)));
  if (templateHeadings.size === 0) return false;
  return recentStyle.commonHeadings.every((entry) => !templateHeadings.has(normalizeHeading(entry.heading)));
}

function missingSignals(input: {
  agentInstructionPaths: string[];
  humanInstructionPaths: string[];
  templates: PullRequestTemplateSummary[];
  workflows: WorkflowSummary[];
  packageMetadata: PackageMetadataSummary[];
}): string[] {
  const missing: string[] = [];
  if (input.agentInstructionPaths.length === 0) missing.push("agent-instructions");
  if (input.humanInstructionPaths.length === 0) missing.push("human-contributor-docs");
  if (input.templates.length === 0) missing.push("pull-request-template");
  if (input.workflows.length === 0) missing.push("ci-workflows");
  if (input.packageMetadata.length === 0) missing.push("package-metadata");
  return missing;
}

function summarizeMaintainerConstraints(input: {
  agentInstructionPaths: string[];
  humanInstructionPaths: string[];
  templates: PullRequestTemplateSummary[];
  workflows: WorkflowSummary[];
  securityPolicyPaths: string[];
  releaseWorkflowPaths: string[];
  releaseDocPaths: string[];
}): string[] {
  const constraints: string[] = [];
  if (input.agentInstructionPaths.length > 0) constraints.push("Follow repo-local agent instructions before generic architect-mcp defaults.");
  if (input.humanInstructionPaths.length > 0) constraints.push("Preserve README/CONTRIBUTING setup and contribution expectations.");
  if (input.templates.length > 0) constraints.push("Preserve pull request template sections and checkboxes before applying recent PR style.");
  if (input.workflows.length > 0) constraints.push("Use repository CI workflow names and package scripts as verification hints.");
  if (input.securityPolicyPaths.length > 0) constraints.push("Do not disclose sensitive security details publicly; follow the security policy.");
  if (input.releaseWorkflowPaths.length > 0 || input.releaseDocPaths.length > 0) constraints.push("Respect release/changelog policy for release-sensitive changes.");
  return constraints;
}
