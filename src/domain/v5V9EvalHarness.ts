import { generateContract } from "./contract.js";
import { resolveStandardsProfile, simulatePolicyGate } from "./v5Standards.js";
import { analyzeRegressionCoverage, listPolicyBundles, validatePolicyBundles } from "./v6Governance.js";
import { compareStandardsProfiles, createArchitectureStrategyMap } from "./v7Strategy.js";
import { runFailureModeDrills, selectReviewPlaybook } from "./v8Automation.js";
import { evaluateScenarioAcceptance, selectLocalOrchestrationRecipe } from "./v9OperatingModel.js";

export type V5V9Stage = "v5" | "v6" | "v7" | "v8" | "v9";

export function runV5V9EvalHarness(stage: V5V9Stage = "v9") {
  const stages = ["v5", "v6", "v7", "v8", "v9"] as const;
  const selected = stages.slice(0, stages.indexOf(stage) + 1);
  const results = selected.flatMap((item) => runStage(item));
  return {
    stage,
    status: results.every((result) => result.passed) ? "pass" : "fail",
    summary: {
      total: results.length,
      passed: results.filter((result) => result.passed).length,
      failed: results.filter((result) => !result.passed).length
    },
    results
  };
}

function runStage(stage: V5V9Stage): Array<{ stage: V5V9Stage; name: string; passed: boolean; detail: string }> {
  const brief = {
    idea: "Local-first MCP standards harness for coding agents",
    stack: { backend: "TypeScript MCP server" },
    verification: ["npm run typecheck", "npm test", "npm run build"]
  };
  if (stage === "v5") {
    const profile = resolveStandardsProfile({ brief });
    const simulation = simulatePolicyGate({ findings: [] });
    return [
      { stage, name: "standards profile resolves", passed: profile.profile === "balanced", detail: profile.profile },
      { stage, name: "policy simulation returns all modes", passed: simulation.simulations.length === 4, detail: `${simulation.simulations.length} modes` }
    ];
  }
  if (stage === "v6") {
    const validation = validatePolicyBundles();
    return [
      { stage, name: "policy bundles validate", passed: validation.valid, detail: validation.errors.join("; ") || `${validation.bundleCount} bundles` },
      { stage, name: "regression coverage reports gaps", passed: Array.isArray(analyzeRegressionCoverage().gaps), detail: "coverage calculated" }
    ];
  }
  if (stage === "v7") {
    const strategy = createArchitectureStrategyMap({ brief });
    return [
      { stage, name: "strategy map connects standards", passed: Array.isArray(strategy.standards), detail: `${strategy.standards.length} standards` },
      { stage, name: "profile comparison includes strict", passed: compareStandardsProfiles().profiles.some((profile) => profile.profile === "strict"), detail: "strict included" }
    ];
  }
  if (stage === "v8") {
    return [
      { stage, name: "failure drills pass", passed: runFailureModeDrills().status === "pass", detail: "drills caught" },
      { stage, name: "playbook selection returns tools", passed: selectReviewPlaybook({ request: "fix auth bug" }).playbook.reviewTools.length > 0, detail: "playbook selected" }
    ];
  }
  const contract = generateContract(brief);
  const recipe = selectLocalOrchestrationRecipe({ request: "review existing repo" });
  const acceptance = evaluateScenarioAcceptance({ verified: true, artifactScores: [{ status: "pass" }] });
  return [
    { stage, name: "orchestration recipe selects tools", passed: recipe.recipe.tools.length > 0, detail: recipe.recipe.id },
    { stage, name: "scenario acceptance passes", passed: acceptance.status === "pass", detail: acceptance.plainEnglish },
    { stage, name: "contract remains local-first", passed: contract.generatedBy.tool === "architect-mcp" && listPolicyBundles().bundles.length > 0, detail: "local contract and bundles available" }
  ];
}
