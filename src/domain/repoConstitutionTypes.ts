import type { FileSummary } from "./types.js";

export type RepoConstitutionArtifact = {
  path: string;
  content: string;
};

export type RepoConstitutionPullRequest = {
  number?: number;
  title?: string;
  author?: string;
  authorAssociation?: string;
  merged?: boolean;
  body?: string;
};

export type RepoConstitutionInput = {
  files?: FileSummary[];
  artifacts?: RepoConstitutionArtifact[];
  recentPullRequests?: RepoConstitutionPullRequest[];
};

export type RepoConstitutionProvenance = {
  source: string;
  path?: string;
  detail: string;
};

export type RepoConstitution = {
  schemaVersion: 1;
  summary: {
    hardSignals: number;
    advisorySignals: number;
    warnings: number;
    missingRecommendedSignals: string[];
    primaryLanguages: string[];
    packageManagers: string[];
    repoShape: string;
  };
  instructions: {
    agentInstructionPaths: string[];
    humanInstructionPaths: string[];
    securityPolicyPaths: string[];
  };
  pullRequests: {
    templates: PullRequestTemplateSummary[];
    recentStyle: RecentPullRequestStyleSummary;
    precedence: string[];
  };
  ci: {
    workflows: WorkflowSummary[];
    labelerConfigPaths: string[];
  };
  release: {
    changelogPaths: string[];
    releaseWorkflowPaths: string[];
    releaseDocPaths: string[];
  };
  packageMetadata: PackageMetadataSummary[];
  maintainerConstraints: string[];
  findings: RepoConstitutionFinding[];
  provenance: RepoConstitutionProvenance[];
  publicSafety: {
    rawContentIncluded: false;
    mutationAllowed: false;
  };
};

export type PullRequestTemplateSummary = {
  path: string;
  hiddenCommentOnly: boolean;
  headings: string[];
  checklistItems: number;
  mentionsLinkedIssues: boolean;
  mentionsReleaseNotes: boolean;
  mentionsVerification: boolean;
};

export type RecentPullRequestStyleSummary = {
  advisory: true;
  sampleSize: number;
  acceptedSamples: number;
  maintainerAuthoredSamples: number;
  botSamplesIgnored: number;
  nonMergedOrUnknownSamplesIgnored: number;
  commonHeadings: Array<{ heading: string; count: number }>;
  checklistObserved: boolean;
  releaseNoteObserved: boolean;
  linkedIssueObserved: boolean;
};

export type WorkflowSummary = {
  path: string;
  name?: string;
  triggers: string[];
};

export type PackageMetadataSummary = {
  path: string;
  manager: string;
  name?: string;
  version?: string;
  scripts?: string[];
};

export type RepoConstitutionFinding = {
  code: "MISSING_PR_TEMPLATE" | "RECENT_PR_STYLE_DIVERGES" | "HIDDEN_OR_SPARSE_PR_TEMPLATE" | "MISSING_CI" | "MISSING_AGENT_INSTRUCTIONS";
  severity: "info" | "warning";
  message: string;
  recommendation: string;
};

export const HUMAN_INSTRUCTION_PATHS = [
  "README.md",
  "CONTRIBUTING.md",
  ".github/CONTRIBUTING.md",
  "docs/CONTRIBUTING.md",
  "CODE_OF_CONDUCT.md",
  ".github/CODE_OF_CONDUCT.md"
];

export const AGENT_INSTRUCTION_PATHS = [
  "AGENTS.md",
  ".github/copilot-instructions.md",
  ".cursor/rules/architecture.mdc"
];

export const SECURITY_POLICY_PATHS = [
  "SECURITY.md",
  ".github/SECURITY.md"
];

export const RELEASE_DOC_PATTERNS = [
  /^RELEASE(?:S)?\.md$/i,
  /^docs\/release[-_a-z0-9]*\.md$/i,
  /^docs\/.*release.*\.md$/i
];
