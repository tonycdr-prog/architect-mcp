export type RepoQualityDecision = "proceed" | "ask_more" | "fix_before_generate" | "block";

export type RepoQualityGateSeverity = "blocker" | "error" | "warning";

export type RepoQualityDimension =
  | "requirements_fit"
  | "simplicity"
  | "maintainability"
  | "security"
  | "ci_tests"
  | "documentation"
  | "agent_readiness"
  | "nontechnical_suitability";

export type RepoQualityFinding = {
  code: string;
  severity: RepoQualityGateSeverity;
  dimension: RepoQualityDimension;
  message: string;
  recommendation: string;
};

export type RepoQualityRubricScore = {
  dimension: RepoQualityDimension;
  score: number;
  status: "pass" | "warn" | "fail";
  evidence: string[];
  improvement: string;
};

export type RepoQualityRequirementsProfile = {
  userLevel: "nontechnical" | "beginner" | "technical";
  goals: string[];
  constraints: string[];
  knownRisks: string[];
  missingQuestions: string[];
  confidence: "low" | "medium" | "high";
};

export type RepoQualityPlan = {
  stack?: string[];
  architecture?: string;
  tradeoffs?: string[];
  files?: string[];
  ciCommands?: string[];
  testDescriptions?: string[];
  docs?: string[];
  envVars?: string[];
  permissions?: string[];
  destructiveCommands?: string[];
  explanations?: string[];
};

export type RepoQualityArtifactSignals = {
  hasReadme?: boolean;
  hasSetupInstructions?: boolean;
  hasEnvExample?: boolean;
  hasAgentsMd?: boolean;
  agentsMdVague?: boolean;
  hasMeaningfulCi?: boolean;
  ciOnlyEchoes?: boolean;
  hasMeaningfulTests?: boolean;
  testsAreTrivial?: boolean;
  hasHardcodedSecrets?: boolean;
  unsafePermissions?: boolean;
  overcomplicatedStack?: boolean;
  underpoweredStack?: boolean;
  jargonHeavy?: boolean;
  explainsTradeoffs?: boolean;
};

export type RepoQualityEvaluationInput = {
  profile?: RepoQualityRequirementsProfile;
  plan?: RepoQualityPlan;
  signals?: RepoQualityArtifactSignals;
};

export type RepoQualityEvaluationResult = {
  decision: RepoQualityDecision;
  stoplight: "green" | "yellow" | "red";
  hardGates: RepoQualityFinding[];
  scores: RepoQualityRubricScore[];
  overallScore: number;
  confidence: "low" | "medium" | "high";
  followUpQuestions: string[];
  antiRewardHackingWarnings: string[];
  nextActions: string[];
};
