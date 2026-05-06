export type StackProfile = {
  frontend?: string;
  backend?: string;
  database?: string;
  auth?: string;
  deployment?: string;
};

export type ProjectBrief = {
  idea: string;
  users?: string;
  coreFlows?: string[];
  dataEntities?: string[];
  stack?: StackProfile;
  constraints?: string[];
  repoLayout?: RepoLayout;
  storage?: string;
  enforcement?: string;
  risk?: string;
  verification?: string[];
};

export type ArchitectureContract = {
  contractVersion: string;
  generatedBy: {
    tool: "architect-mcp";
    version: string;
    generatedAt: string;
  };
  name: string;
  purpose: string;
  stack: StackProfile;
  stackPacks: StackPack[];
  directories: DirectoryRule[];
  fileRules: FileRule[];
  moduleBoundaries: string[];
  testingExpectations: string[];
  agentInstructions: string[];
  foundationPacks: FoundationPack[];
  archetype?: AppArchetype;
};

export type AppArchetype =
  | "saas-dashboard"
  | "marketplace"
  | "internal-admin"
  | "mobile-field-app"
  | "ai-workflow-tool"
  | "content-docs-site"
  | "custom-app";

export type FoundationPack = {
  id: "agent-harness" | "testing" | "repo-structure" | "ci-gates" | "repo-hygiene";
  name: string;
  version: string;
  rationale: string;
  sources: StackPackSource[];
  rules: string[];
  artifacts: string[];
  reviewQuestions: string[];
};

export type RepoLayout = {
  pathMap: Record<string, string[]>;
};

export type StackPack = {
  id: string;
  name: string;
  version: string;
  rationale: string;
  sources: StackPackSource[];
  appliesTo: Array<keyof StackProfile>;
  aliases?: string[];
  directories: DirectoryRule[];
  fileRules: FileRule[];
  moduleBoundaries: string[];
  testingExpectations: string[];
  agentInstructions: string[];
};

export type StackPackSource = {
  label: string;
  url?: string;
  note?: string;
};

export type DirectoryRule = {
  path: string;
  purpose: string;
  required: boolean;
};

export type FileRule = {
  name: string;
  rule: string;
  severity: "error" | "warning";
  trigger?: string;
  recommendation?: string;
  appliesToPaths?: string[];
  goodExample?: string;
  badExample?: string;
  triggerKind?: RuleTriggerKind;
  detectors?: RuleDetector[];
};

export type RuleTriggerKind =
  | "line-threshold"
  | "import-boundary"
  | "direct-db-access"
  | "env-access"
  | "client-boundary"
  | "route-thinness"
  | "migration-discipline"
  | "hosted-filesystem"
  | "auth-boundary"
  | "payment-boundary"
  | "test-policy"
  | "validation-boundary"
  | "ai-tool-safety"
  | "manual-review";

export type RuleDetector = {
  kind: RuleTriggerKind;
  description: string;
};

export type FileSummary = {
  path: string;
  lines?: number;
  bytes?: number;
  imports?: string[];
  hasUseClient?: boolean;
  envAccesses?: string[];
  hasDirectDbAccess?: boolean;
};
