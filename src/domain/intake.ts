import { generateRepoArtifacts, generateScaffoldPlan } from "./artifacts.js";
import { archetypeQuestions, inferAppArchetype } from "./archetypes.js";
import { generateBuildPlan } from "./buildPlan.js";
import { generateContract, renderContractMarkdown } from "./contract.js";
import { listFoundationPacks } from "./foundationPacks.js";
import { getBlockers, getChallenges, scoreReadiness, scoreSpecCompleteness } from "./intakeAnalysis.js";
import { getNextIntakeQuestionForBrief, QUESTIONS } from "./intakeQuestions.js";
import { resolveStackPacks } from "./stackPacks.js";
import type { GrillMeOptions, GrillMeResult, IntakeAnswer, ProjectBrief } from "./types.js";

export function getNextIntakeQuestion(brief: ProjectBrief) {
  return getNextIntakeQuestionForBrief(brief);
}

export function grillProjectBrief(brief: ProjectBrief) {
  return grillMe(brief, {
    includeContract: false,
    includeArtifacts: false
  });
}

export function grillMe(brief: ProjectBrief, options: GrillMeOptions = {}): GrillMeResult {
  const updatedBrief = options.answer ? applyIntakeAnswer(brief, options.answer) : brief;
  const nextQuestion = getNextIntakeQuestion(updatedBrief);
  const missingFields = QUESTIONS
    .filter((question) => !Boolean(updatedBrief[question.id as keyof ProjectBrief]))
    .map((question) => question.id);
  const coveredFields = QUESTIONS
    .filter((question) => Boolean(updatedBrief[question.id as keyof ProjectBrief]))
    .map((question) => question.id);
  const blockers = getBlockers(updatedBrief);
  const challenges = getChallenges(updatedBrief, missingFields);
  const archetype = inferAppArchetype(updatedBrief);
  const foundationPacks = listFoundationPacks();
  for (const question of archetypeQuestions(archetype).slice(0, 2)) {
    challenges.push({
      field: "archetype",
      severity: "pressure-test",
      question,
      whyItMatters: "Archetype-specific questions keep generated files aligned to the app's real workflows."
    });
  }
  const challengeBlockers = challenges
    .filter((challenge) => challenge.severity === "blocker" && !missingFields.includes(challenge.field as typeof missingFields[number]))
    .map((challenge) => `Unresolved architecture question: ${challenge.question}`);
  const allBlockers = [...blockers, ...challengeBlockers];
  const specCompleteness = scoreSpecCompleteness(updatedBrief, challenges);
  const answeredCount = QUESTIONS.length - missingFields.length;
  const readinessScore = scoreReadiness(answeredCount, allBlockers.length, challenges.length);
  const ready = readinessScore >= 75 && allBlockers.length === 0;
  const selectedStackPacks = resolveStackPacks(updatedBrief.stack ?? {}, options.stackPackIds ?? []).map((pack) => pack.id);
  const includeContract = options.includeContract ?? ready;
  const contract = includeContract ? generateContract(updatedBrief, options.stackPackIds ?? []) : undefined;

  return {
    readinessScore,
    specCompleteness,
    ready,
    phase: ready ? "contract" : "intake",
    missingFields,
    coveredFields,
    blockers: allBlockers,
    challenges,
    nextQuestion,
    selectedStackPacks,
    contract,
    markdown: contract ? renderContractMarkdown(contract) : undefined,
    artifacts: contract && options.includeArtifacts ? generateRepoArtifacts(contract, updatedBrief) : undefined,
    scaffoldPlan: contract ? generateScaffoldPlan(contract) : undefined,
    buildPlan: generateBuildPlan(updatedBrief),
    archetype,
    foundationPacks,
    updatedBrief,
    instruction: ready
      ? "Generate or commit the architecture contract, then review the repo before implementation."
      : "Answer the blocker and pressure-test questions concretely. Do not begin implementation until the brief covers users, core flows, stack, repo layout, risks, and verification."
  };
}

export function applyIntakeAnswer(brief: ProjectBrief, answer: IntakeAnswer): ProjectBrief {
  if (answer.field === "stack") {
    return {
      ...brief,
      stack: {
        ...(brief.stack ?? {}),
        ...(typeof answer.value === "object" && !Array.isArray(answer.value) ? answer.value as ProjectBrief["stack"] : {})
      }
    };
  }

  return {
    ...brief,
    [answer.field]: answer.value
  };
}
