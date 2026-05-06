import type { RepoQualityArtifactSignals, RepoQualityDecision, RepoQualityDimension, RepoQualityEvaluationInput, RepoQualityFinding, RepoQualityPlan, RepoQualityRequirementsProfile, RepoQualityRubricScore } from "./repoQualityEvalTypes.js";

const dimensions: RepoQualityDimension[] = ["requirements_fit", "simplicity", "maintainability", "security", "ci_tests", "documentation", "agent_readiness", "nontechnical_suitability"];

export function buildQualityRequirementsProfile(input: {
  userLevel?: RepoQualityRequirementsProfile["userLevel"];
  goals?: string[];
  constraints?: string[];
  answers?: string[];
  stackPreference?: string;
} = {}): RepoQualityRequirementsProfile {
  const goals = input.goals?.length ? input.goals : inferGoals(input.answers ?? []);
  const constraints = [...(input.constraints ?? []), ...stackPreferenceConstraints(input.stackPreference)];
  const text = [...goals, ...constraints, ...(input.answers ?? []), input.stackPreference ?? ""].join(" ").toLowerCase();
  const missingQuestions = [
    ...(!goals.length ? ["What outcome should the app produce for the user?"] : []),
    ...(!/user|customer|admin|team|owner/.test(text) ? ["Who will use this, and how technical are they?"] : []),
    ...(!/data|store|save|database|local|account|auth|login/.test(text) ? ["What data must be stored, and does it need accounts or login?"] : []),
    ...(!/deploy|host|local|mobile|web|desktop/.test(text) ? ["Where should this run first: local, web, mobile, or hosted?"] : []),
    ...(!/success|done|test|verify|acceptance/.test(text) ? ["What would prove the generated repo is good enough?"] : [])
  ];
  return {
    userLevel: input.userLevel ?? (/non.?technical|vibe|beginner|novice/.test(text) ? "beginner" : "technical"),
    goals,
    constraints,
    knownRisks: riskTerms(text),
    missingQuestions,
    confidence: missingQuestions.length >= 3 ? "low" : missingQuestions.length ? "medium" : "high"
  };
}

export function evaluateRepoPlanQuality(input: RepoQualityEvaluationInput = {}) {
  return evaluateQuality(input, "plan");
}

export function auditGeneratedRepoQuality(input: RepoQualityEvaluationInput = {}) {
  return evaluateQuality(input, "repo");
}

export function suggestQualityFollowUpQuestions(input: RepoQualityEvaluationInput = {}) {
  const profile = input.profile ?? buildQualityRequirementsProfile();
  const result = evaluateQuality(input, "plan");
  return {
    confidence: result.confidence,
    questions: result.followUpQuestions.slice(0, 3),
    reason: profile.confidence === "low" ? "Requirements are underspecified." : result.hardGates.length ? "Hard gates need explicit correction." : "Questions target the weakest rubric areas."
  };
}

export function runRepoQualityEvalScenarios() {
  const scenarios = [
    { name: "blocks committed secrets", passed: evaluateRepoPlanQuality({ signals: { hasHardcodedSecrets: true } }).decision === "block" },
    { name: "fails fake CI despite green workflow", passed: evaluateRepoPlanQuality({ signals: { hasMeaningfulCi: false, ciOnlyEchoes: true } }).hardGates.some((gate) => gate.code === "RQG003_FAKE_CI") },
    { name: "fails trivial tests", passed: evaluateRepoPlanQuality({ signals: { hasMeaningfulTests: false, testsAreTrivial: true } }).hardGates.some((gate) => gate.code === "RQG004_FAKE_TESTS") },
    { name: "asks more when requirements are vague", passed: evaluateRepoPlanQuality({ profile: buildQualityRequirementsProfile({ goals: [] }) }).decision === "ask_more" },
    { name: "passes a boring maintainable repo plan", passed: evaluateRepoPlanQuality(goodFixture()).decision === "proceed" },
    { name: "warns against CI reward hacking", passed: evaluateRepoPlanQuality({ signals: { hasMeaningfulCi: true, testsAreTrivial: true } }).antiRewardHackingWarnings.some((warning) => /CI/.test(warning)) }
  ];
  return {
    status: scenarios.every((scenario) => scenario.passed) ? "pass" : "fail",
    summary: {
      total: scenarios.length,
      passed: scenarios.filter((scenario) => scenario.passed).length,
      failed: scenarios.filter((scenario) => !scenario.passed).length
    },
    scenarios
  };
}

function evaluateQuality(input: RepoQualityEvaluationInput, phase: "plan" | "repo") {
  const profile = input.profile ?? buildQualityRequirementsProfile();
  const plan = input.plan ?? {};
  const signals = input.signals ?? inferSignals(plan);
  const hardGates = hardGateFindings(profile, plan, signals, phase);
  const scores = dimensions.map((dimension) => scoreDimension(dimension, profile, plan, signals));
  const overallScore = Math.round(scores.reduce((sum, score) => sum + score.score, 0) / scores.length);
  const decision = decide(profile, hardGates, overallScore);
  return {
    decision,
    stoplight: decision === "block" || decision === "fix_before_generate" ? "red" : decision === "ask_more" || overallScore < 80 ? "yellow" : "green",
    hardGates,
    scores,
    overallScore,
    confidence: confidence(profile, scores),
    followUpQuestions: followUpQuestions(profile, scores, hardGates),
    antiRewardHackingWarnings: rewardHackingWarnings(plan, signals, scores),
    nextActions: nextActions(decision, hardGates, scores)
  };
}

function hardGateFindings(profile: RepoQualityRequirementsProfile, plan: RepoQualityPlan, signals: RepoQualityArtifactSignals, phase: "plan" | "repo"): RepoQualityFinding[] {
  const findings: RepoQualityFinding[] = [];
  const requireEvidence = phase === "repo";
  if (signals.hasHardcodedSecrets) findings.push(gate("RQG001_COMMITTED_SECRET", "blocker", "security", "Secrets or secret-like values are present.", "Remove the secret, rotate it, and use environment variables plus .env.example."));
  if ((plan.envVars?.length ?? 0) > 0 && (signals.hasEnvExample === false || (requireEvidence && signals.hasEnvExample !== true))) findings.push(gate("RQG002_ENV_EXAMPLE_MISSING", "error", "documentation", "Environment variables are needed but .env.example is missing.", "Add .env.example with names and safe placeholder values."));
  if (signals.hasMeaningfulCi === false || signals.ciOnlyEchoes || (requireEvidence && signals.hasMeaningfulCi !== true)) findings.push(gate("RQG003_FAKE_CI", "error", "ci_tests", "CI does not run meaningful checks.", "CI must run real typecheck, tests, lint/build, or equivalent project checks."));
  if (signals.hasMeaningfulTests === false || signals.testsAreTrivial || (requireEvidence && signals.hasMeaningfulTests !== true)) findings.push(gate("RQG004_FAKE_TESTS", "error", "ci_tests", "Tests are missing, trivial, or fake.", "Add behavior or integration tests that can fail for real regressions."));
  if ((plan.destructiveCommands?.length ?? 0) > 0) findings.push(gate("RQG005_DESTRUCTIVE_COMMAND", "blocker", "security", "Plan includes destructive commands.", "Require explicit user approval and a rollback note before running destructive commands."));
  if (signals.unsafePermissions || (plan.permissions ?? []).some((permission) => /\*|admin|write all|full access/i.test(permission))) findings.push(gate("RQG006_UNSAFE_PERMISSIONS", "blocker", "security", "Permissions are too broad or unsafe.", "Use least-privilege scopes and explain why each permission is needed."));
  if (signals.hasSetupInstructions === false || (requireEvidence && (signals.hasReadme !== true || signals.hasSetupInstructions !== true))) findings.push(gate("RQG007_SETUP_DOCS_MISSING", "error", "documentation", "Setup instructions are missing.", "Add README setup, run, test, and deployment notes for the target user."));
  if (signals.hasAgentsMd === false || signals.agentsMdVague || (requireEvidence && signals.hasAgentsMd !== true)) findings.push(gate("RQG008_AGENTS_MD_WEAK", "error", "agent_readiness", "AGENTS.md is missing or vague.", "Add repo-specific commands, boundaries, verification rules, and non-goals for future agents."));
  if (profile.userLevel !== "technical" && signals.jargonHeavy) findings.push(gate("RQG009_JARGON_FOR_NOVICE", "error", "nontechnical_suitability", "User-facing explanation is too jargon-heavy.", "Use plain language and explain tradeoffs without assuming professional terminology."));
  return findings;
}

function scoreDimension(dimension: RepoQualityDimension, profile: RepoQualityRequirementsProfile, plan: RepoQualityPlan, signals: RepoQualityArtifactSignals): RepoQualityRubricScore {
  let score = 80;
  const evidence: string[] = [];
  if (dimension === "requirements_fit") {
    score = profile.confidence === "high" ? 90 : profile.confidence === "medium" ? 70 : 45;
    evidence.push(`${profile.missingQuestions.length} missing requirement questions`);
  }
  if (dimension === "simplicity") {
    score = signals.overcomplicatedStack ? 45 : signals.underpoweredStack ? 60 : 88;
    evidence.push(`stack=${(plan.stack ?? []).join(", ") || "not supplied"}`);
  }
  if (dimension === "maintainability") {
    score = (plan.files?.length ?? 0) > 0 && /boundary|feature|module|service|component/i.test(plan.architecture ?? "") ? 88 : 62;
    evidence.push(plan.architecture ?? "architecture not supplied");
  }
  if (dimension === "security") {
    score = signals.hasHardcodedSecrets || signals.unsafePermissions ? 20 : (plan.permissions?.length ?? 0) > 0 ? 72 : 85;
    evidence.push(signals.hasHardcodedSecrets ? "secret signal present" : "no secret signal supplied");
  }
  if (dimension === "ci_tests") {
    score = signals.hasMeaningfulCi && signals.hasMeaningfulTests && !signals.testsAreTrivial ? 90 : signals.hasMeaningfulCi || signals.hasMeaningfulTests ? 60 : 30;
    evidence.push(`ci=${signals.hasMeaningfulCi === true}`, `tests=${signals.hasMeaningfulTests === true}`);
  }
  if (dimension === "documentation") {
    score = signals.hasReadme && signals.hasSetupInstructions && ((plan.envVars?.length ?? 0) === 0 || signals.hasEnvExample) ? 90 : 55;
    evidence.push(`readme=${signals.hasReadme === true}`, `setup=${signals.hasSetupInstructions === true}`, `envExample=${signals.hasEnvExample === true}`);
  }
  if (dimension === "agent_readiness") {
    score = signals.hasAgentsMd && !signals.agentsMdVague ? 88 : 35;
    evidence.push(`agentsMd=${signals.hasAgentsMd === true}`);
  }
  if (dimension === "nontechnical_suitability") {
    score = profile.userLevel === "technical" ? 85 : signals.jargonHeavy ? 45 : signals.explainsTradeoffs ? 88 : 65;
    evidence.push(`userLevel=${profile.userLevel}`, `tradeoffs=${signals.explainsTradeoffs === true}`);
  }
  return {
    dimension,
    score,
    status: score >= 80 ? "pass" : score >= 60 ? "warn" : "fail",
    evidence,
    improvement: improvementFor(dimension)
  };
}

function decide(profile: RepoQualityRequirementsProfile, gates: RepoQualityFinding[], score: number): RepoQualityDecision {
  if (gates.some((gateItem) => gateItem.severity === "blocker")) return "block";
  if (gates.length > 0) return "fix_before_generate";
  if (profile.confidence === "low") return "ask_more";
  if (score >= 80) return "proceed";
  return "ask_more";
}

function followUpQuestions(profile: RepoQualityRequirementsProfile, scores: RepoQualityRubricScore[], gates: RepoQualityFinding[]): string[] {
  if (gates.length) return gates.slice(0, 2).map((gateItem) => `Before proceeding, can I fix this: ${gateItem.message}`);
  const weak = scores.filter((score) => score.status !== "pass").map((score) => score.dimension);
  return [...profile.missingQuestions, ...weak.map(questionForDimension)].slice(0, 3);
}

function rewardHackingWarnings(plan: RepoQualityPlan, signals: RepoQualityArtifactSignals, scores: RepoQualityRubricScore[]): string[] {
  const warnings: string[] = [];
  if (signals.hasMeaningfulCi && (signals.testsAreTrivial || signals.hasMeaningfulTests === false)) warnings.push("CI passing is not enough because the tests are weak or fake.");
  if (signals.overcomplicatedStack) warnings.push("A complex stack should not score well unless the requirements justify it.");
  if (signals.underpoweredStack) warnings.push("A simple stack should not score well if it cannot meet the user's goals.");
  if ((plan.explanations ?? []).some((item) => /user asked|because user said/i.test(item)) && scores.some((score) => score.dimension === "security" && score.status !== "pass")) warnings.push("User preference cannot override security or maintainability hard gates.");
  return warnings;
}

function nextActions(decision: RepoQualityDecision, gates: RepoQualityFinding[], scores: RepoQualityRubricScore[]): string[] {
  if (decision === "block" || decision === "fix_before_generate") return gates.slice(0, 3).map((gateItem) => gateItem.recommendation);
  if (decision === "ask_more") return scores.filter((score) => score.status !== "pass").slice(0, 3).map((score) => score.improvement);
  return ["Proceed, then audit the generated repo with audit_generated_repo_quality before presenting it as done."];
}

function inferSignals(plan: RepoQualityPlan): RepoQualityArtifactSignals {
  const hasCiCommands = (plan.ciCommands?.length ?? 0) > 0;
  const hasTests = (plan.testDescriptions?.length ?? 0) > 0;
  const hasDocs = (plan.docs?.length ?? 0) > 0;
  return {
    hasReadme: hasDocs ? plan.docs?.some((doc) => /readme/i.test(doc)) : undefined,
    hasSetupInstructions: hasDocs ? plan.docs?.some((doc) => /setup|install|run/i.test(doc)) : undefined,
    hasEnvExample: hasDocs ? plan.docs?.some((doc) => /\.env\.example/i.test(doc)) : undefined,
    hasAgentsMd: hasDocs ? plan.docs?.some((doc) => /agents\.md/i.test(doc)) : undefined,
    agentsMdVague: false,
    hasMeaningfulCi: hasCiCommands ? plan.ciCommands?.some((command) => /test|typecheck|lint|build|cargo test|go test/i.test(command)) : undefined,
    ciOnlyEchoes: hasCiCommands ? plan.ciCommands?.every((command) => /^echo\b/i.test(command.trim())) : undefined,
    hasMeaningfulTests: hasTests ? !plan.testDescriptions?.every((test) => /trivial|placeholder|true/i.test(test)) : undefined,
    testsAreTrivial: hasTests ? plan.testDescriptions?.some((test) => /expect\(true\)|placeholder|trivial/i.test(test)) : undefined,
    hasHardcodedSecrets: plan.envVars?.some((value) => /=\s*(sk-|pk_|ghp_|xoxb-|AKIA)/i.test(value)),
    unsafePermissions: plan.permissions?.some((permission) => /\*|admin|full access|write all/i.test(permission)),
    overcomplicatedStack: (plan.stack?.length ?? 0) > 5,
    underpoweredStack: /auth|payment|team|account/i.test([...profileless(plan), ...(plan.explanations ?? [])].join(" ")) && (plan.stack?.length ?? 0) <= 1,
    jargonHeavy: plan.explanations?.some((item) => /\bCQRS|event sourcing|hexagonal|idempotency|eventual consistency\b/i.test(item)),
    explainsTradeoffs: (plan.tradeoffs?.length ?? 0) > 0
  };
}

function profileless(plan: RepoQualityPlan): string[] {
  return [plan.architecture ?? "", ...(plan.files ?? []), ...(plan.docs ?? [])];
}

function inferGoals(answers: string[]): string[] {
  return answers.filter((answer) => /build|create|manage|track|help|app|tool/i.test(answer)).slice(0, 5);
}

function riskTerms(text: string): string[] {
  return ["auth", "payment", "secret", "database", "delete", "admin", "public"].filter((term) => text.includes(term));
}

function stackPreferenceConstraints(stackPreference: string | undefined): string[] {
  if (!stackPreference) return [];
  const stack = stackPreference.toLowerCase();
  const constraints: string[] = [`Stack preference: ${stackPreference}`];
  if (/supabase/.test(stack)) constraints.push("Supabase plans must explain auth boundaries, database access boundaries, row-level security, and deployment environment variables.");
  if (/next/.test(stack)) constraints.push("Next.js plans must separate server/client boundaries and avoid leaking secrets into client components.");
  if (/stripe/.test(stack)) constraints.push("Stripe plans must include webhook signature verification, idempotency, and entitlement boundaries.");
  return constraints;
}

function gate(code: string, severity: RepoQualityFinding["severity"], dimension: RepoQualityDimension, message: string, recommendation: string): RepoQualityFinding {
  return { code, severity, dimension, message, recommendation };
}

function confidence(profile: RepoQualityRequirementsProfile, scores: RepoQualityRubricScore[]): RepoQualityRequirementsProfile["confidence"] {
  if (profile.confidence === "low" || scores.some((score) => score.status === "fail")) return "low";
  if (profile.confidence === "medium" || scores.some((score) => score.status === "warn")) return "medium";
  return "high";
}

function questionForDimension(dimension: RepoQualityDimension): string {
  return {
    requirements_fit: "Which user workflow matters most for the first version?",
    simplicity: "Is a boring standard stack acceptable if it meets the goal?",
    maintainability: "Which parts should future agents treat as separate modules?",
    security: "What auth, permission, or secret-handling risk should be protected first?",
    ci_tests: "What behavior should tests prove, beyond CI turning green?",
    documentation: "Who needs to run this repo, and what setup detail would block them?",
    agent_readiness: "What should future coding agents avoid changing?",
    nontechnical_suitability: "Should explanations be written for a non-technical user?"
  }[dimension];
}

function improvementFor(dimension: RepoQualityDimension): string {
  return {
    requirements_fit: "Ask one focused question about users, data, runtime, or success proof.",
    simplicity: "Justify stack complexity against the user's goals.",
    maintainability: "Name module boundaries and ownership before generation.",
    security: "Remove unsafe defaults and use least-privilege, env placeholders, and explicit trust boundaries.",
    ci_tests: "Add real checks and tests that can fail for meaningful regressions.",
    documentation: "Add README setup/run/test docs and .env.example where env vars exist.",
    agent_readiness: "Add AGENTS.md with commands, boundaries, non-goals, and proof requirements.",
    nontechnical_suitability: "Rewrite user-facing explanations in plain language with clear tradeoffs."
  }[dimension];
}

function goodFixture(): RepoQualityEvaluationInput {
  return {
    profile: buildQualityRequirementsProfile({
      userLevel: "beginner",
      goals: ["Build a small customer intake app for admin users"],
      constraints: ["Runs as a web app", "Stores customer data", "Success means admins can create and review intakes"]
    }),
    plan: {
      stack: ["React", "Express", "Postgres"],
      architecture: "Feature modules with server-owned repositories and UI components separated from data access.",
      files: ["src/features/intake", "src/server/repositories", "tests/intake.test.ts"],
      ciCommands: ["npm run typecheck", "npm test", "npm run build"],
      testDescriptions: ["Creates an intake and validates required fields"],
      docs: ["README.md setup/run/test", ".env.example", "AGENTS.md"],
      envVars: ["DATABASE_URL"],
      tradeoffs: ["Boring stack is easier to maintain than a distributed architecture."],
      explanations: ["This uses common tools so another developer can pick it up later."]
    },
    signals: {
      hasReadme: true,
      hasSetupInstructions: true,
      hasEnvExample: true,
      hasAgentsMd: true,
      agentsMdVague: false,
      hasMeaningfulCi: true,
      hasMeaningfulTests: true,
      testsAreTrivial: false,
      hasHardcodedSecrets: false,
      unsafePermissions: false,
      explainsTradeoffs: true,
      jargonHeavy: false
    }
  };
}
