import type { FoundryDecisionRoute } from "./foundryDecisionLedgerTypes.js";
import type { FoundryEvidenceInventoryInput, FoundrySuppressionCategory } from "./foundryEvidenceTypes.js";
import type { RepoConstitutionInput } from "./repoConstitutionTypes.js";

export type FoundryEvalCorpusRepoSize = "large" | "small" | "tiny";
export type FoundryEvalCorpusRepoType =
  | "web_framework"
  | "python_framework"
  | "rust_cli"
  | "node_library"
  | "mcp_server";

export type FoundryEvalCorpusCase = {
  id: string;
  repo: {
    slug: string;
    size: FoundryEvalCorpusRepoSize;
    type: FoundryEvalCorpusRepoType;
    language: string;
  };
  expectedRoutes: FoundryDecisionRoute[];
  expectedNoisePatterns: string[];
  regressionIssues: number[];
  files?: RepoConstitutionInput["files"];
  artifacts?: RepoConstitutionInput["artifacts"];
  recentPullRequests?: RepoConstitutionInput["recentPullRequests"];
  evidence: Omit<FoundryEvidenceInventoryInput, "repoConstitution">;
  verificationHints?: string[];
};

export type FoundryEvalCorpusInput = {
  caseIds?: string[];
  includePreviews?: boolean;
};

export type FoundryEvalRegressionAssertion = {
  issue: number;
  passed: boolean;
  evidence: string;
};

export type FoundryEvalCorpusResult = {
  id: string;
  repo: FoundryEvalCorpusCase["repo"];
  status: "pass" | "fail";
  expectedRoutes: FoundryDecisionRoute[];
  missingExpectedRoutes: FoundryDecisionRoute[];
  routes: Record<string, number>;
  previewKinds: Record<string, number>;
  suppressionCategories: FoundrySuppressionCategory[];
  regressionIssues: number[];
  declaredRegressionIssues: number[];
  missingDeclaredRegressionIssues: number[];
  regressionAssertions: FoundryEvalRegressionAssertion[];
  expectedNoisePatterns: string[];
  publicSafety: FoundryEvalPublicSafety;
  noMutation: {
    passed: boolean;
    serverWritesPerformed: 0;
    mutationAllowed: false;
  };
};

export type FoundryEvalPublicSafety = {
  passed: boolean;
  leaks: string[];
  rawPayloadsIncluded: false;
  rawRepoContentIncluded: false;
  localPathsIncluded: false;
  tokenValuesIncluded: false;
};

export type FoundryEvalCorpusReport = {
  schemaVersion: 1;
  status: "pass" | "partial" | "fail";
  summary: {
    totalCases: number;
    passed: number;
    failed: number;
    byRepoSize: Record<string, number>;
    byRoute: Record<string, number>;
    previewKinds: Record<string, number>;
    suppressionCategories: FoundrySuppressionCategory[];
    regressionIssuesCovered: number[];
    selectedSubset: boolean;
    unknownCaseIds: string[];
    offlineNetworkRequired: false;
    liveSmokeAvailable: true;
    serverWritesPerformed: 0;
  };
  requirements: {
    offlineFixtures: boolean;
    optionalLiveSmoke: boolean;
    routeCoverage: Record<FoundryDecisionRoute, boolean>;
    publicSafety: boolean;
    noMutation: boolean;
    regressionIssues: Record<string, boolean>;
  };
  manifest: {
    cases: Array<Pick<FoundryEvalCorpusCase, "id" | "expectedRoutes" | "expectedNoisePatterns" | "regressionIssues"> & {
      repo: FoundryEvalCorpusCase["repo"];
    }>;
  };
  results: FoundryEvalCorpusResult[];
  liveSmoke: {
    mode: "optional_read_only";
    networkRequired: true;
    command: string;
    selectedPublicRepos: string[];
    noMutationChecks: string[];
  };
  publicSafety: FoundryEvalPublicSafety;
};
