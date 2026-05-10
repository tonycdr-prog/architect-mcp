export type ReviewViolation = {
  code: FindingCode;
  confidence: "high" | "medium" | "low";
  severity: "error" | "warning";
  path?: string;
  message: string;
  recommendation: string;
};

export type FindingCode =
  | "ARCH001_OVERSIZED_FILE"
  | "ARCH002_UI_DB_ACCESS"
  | "ARCH003_CLIENT_SERVER_LEAK"
  | "ARCH004_THIN_CONTROLLER"
  | "ARCH005_MISSING_DIRECTORY"
  | "ARCH006_MIXED_CONCERNS"
  | "ARCH007_SHARED_IMPORTS_FEATURE"
  | "ARCH008_ENV_SCATTER"
  | "ARCH009_SERVER_ENV_IN_UI"
  | "ARCH010_ROUTE_WITHOUT_FEATURES"
  | "ARCH011_TRANSPORT_LEAK"
  | "ARCH012_MIGRATION_DISCIPLINE"
  | "ARCH013_SUPABASE_SPLIT"
  | "ARCH014_PACK_RULE"
  | "ARCH015_PLAN_MONOLITH_RISK"
  | "ARCH016_PLAN_MISSING_BOUNDARY"
  | "ARCH017_PLAN_MISSING_HARNESS"
  | "ARCH018_BUILD_PLAN_ORDER"
  | "ARCH019_BUILD_PLAN_VERIFICATION"
  | "ARCH020_IMPLEMENTATION_IGNORED_PLAN"
  | "ARCH021_SPEC_INCOMPLETE"
  | "ARCH022_IMPORT_GRAPH_BOUNDARY"
  | "ARCH023_STACK_PACK_CANDIDATE"
  | "ARCH024_AGENT_HARNESS"
  | "ARCH025_TYPE_SCHEMA_AGGREGATION"
  | "ARCH026_REPO_HYGIENE";

export type ReviewMode = "strict" | "summary" | "ci" | "migration" | "audit";

export type ReviewOptions = {
  mode?: ReviewMode;
  ignorePatterns?: string[];
  baseline?: ReviewBaseline;
  maxDetailedFindings?: number;
  summarizeLineWarningsBelow?: number;
  gate?: ReviewGateOptions;
};

export type ReviewGateOptions = {
  maxErrors?: number;
  maxWarnings?: number;
  minScore?: number;
};

export type ReviewBaseline = {
  findings: BaselineFinding[];
};

export type BaselineFinding = {
  code: FindingCode;
  path?: string;
  message?: string;
  status?: "baseline" | "accepted";
  reason?: string;
};

export type ReviewLifecycle = {
  newFindings: ReviewViolation[];
  baselineFindings: ReviewViolation[];
  acceptedFindings: ReviewViolation[];
  resolvedFindings: BaselineFinding[];
};

export type ReviewReport = {
  score: number;
  grade: "A" | "B" | "C" | "D" | "F";
  mode: ReviewMode;
  gate: ReviewGate;
  summary: {
    errors: number;
    warnings: number;
    shown: number;
    suppressed: number;
    noiseSuppressed: number;
    baselineSuppressed: number;
  };
  groups: ReviewGroup[];
  priorityFindings: ReviewViolation[];
  violations: ReviewViolation[];
};

export type ReviewGate = {
  status: "pass" | "warn" | "fail";
  reason: string;
  thresholds: Required<ReviewGateOptions>;
  lifecycle?: {
    acceptedWithoutReason: number;
    newHighConfidenceErrors: number;
  };
};

export type ReviewGroup = {
  key: string;
  count: number;
  severity: "error" | "warning";
  samplePaths: string[];
  recommendation: string;
};
