import type { AppArchetype, ArchitectureContract, FoundationPack, ProjectBrief } from "./coreTypes.js";

export type GrillMeOptions = {
  stackPackIds?: string[];
  includeContract?: boolean;
  includeArtifacts?: boolean;
  answer?: IntakeAnswer;
};

export type IntakeAnswer = {
  field: keyof ProjectBrief;
  value: ProjectBrief[keyof ProjectBrief];
};

export type GrillMeResult = {
  readinessScore: number;
  specCompleteness: SpecCompleteness;
  ready: boolean;
  phase: "intake" | "contract" | "implementation";
  missingFields: string[];
  coveredFields: string[];
  blockers: string[];
  challenges: GrillMeChallenge[];
  nextQuestion: {
    id: string;
    question: string;
    recommendedAnswer: string;
  };
  selectedStackPacks: string[];
  contract?: ArchitectureContract;
  markdown?: string;
  artifacts?: RepoArtifact[];
  scaffoldPlan?: ScaffoldPlanItem[];
  buildPlan: BuildPlan;
  archetype: AppArchetype;
  foundationPacks: FoundationPack[];
  updatedBrief: ProjectBrief;
  instruction: string;
};

export type GrillMeChallenge = {
  field: string;
  severity: "blocker" | "pressure-test";
  question: string;
  whyItMatters: string;
};

export type RepoArtifact = {
  path: string;
  description: string;
  content: string;
};

export type ScaffoldPlanItem = {
  path: string;
  action: "create-directory" | "create-file" | "verify";
  rationale: string;
};

export type BuildPlan = {
  archetype: AppArchetype;
  slices: BuildPlanSlice[];
};

export type BuildPlanSlice = {
  id: string;
  title: string;
  order: number;
  goal: string;
  inputs: string[];
  outputs: string[];
  allowedDirectories: string[];
  forbiddenFiles: string[];
  files: string[];
  checks: string[];
  stopAfter: string;
};

export type BuildPlanReviewOptions = {
  requireHarness?: boolean;
  allowedChecks?: string[];
  contract?: ArchitectureContract;
};

export type SpecCompleteness = {
  score: number;
  checks: Array<{
    id: string;
    status: "pass" | "warn" | "fail";
    summary: string;
  }>;
};

export type ProposedFilePlan = {
  files: ProposedFile[];
};

export type ProposedFile = {
  path: string;
  purpose: string;
  responsibilities?: string[];
};
