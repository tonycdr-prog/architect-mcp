import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { generateRepoArtifacts } from "./artifacts.js";
import { validateRepoArtifacts } from "./artifactValidation.js";
import { generateContract } from "./contract.js";
import { validateArchitectureContract } from "./contractValidation.js";
import { ARCHITECT_MCP_BRIEF, runArchitectSelfReview } from "./selfReview.js";
import { validateFoundationPacks } from "./foundationPacks.js";
import { validateStackPacks } from "./stackPacks.js";
import { validatePolicyBundles } from "./v6Governance.js";
import { runV5V9EvalHarness } from "./v5V9EvalHarness.js";
import { runV10EvalHarness } from "./v10Productization.js";
import { runRepoQualityEvalScenarios } from "./repoQualityEval.js";
import { classifyToolPolicy } from "./toolPolicy.js";
import { registeredArchitectureToolNames } from "../tools/toolRegistry.js";

export type McpReadinessReport = {
  ready: boolean;
  checks: ReadinessCheck[];
};

export type ReadinessCheck = {
  name: string;
  status: "pass" | "warn" | "fail";
  summary: string;
};

export async function createMcpReadinessReport(): Promise<McpReadinessReport> {
  const checks: ReadinessCheck[] = [];
  const packValidation = validateStackPacks();
  checks.push({
    name: "pack validation",
    status: packValidation.valid ? "pass" : "fail",
    summary: packValidation.valid ? `${packValidation.packCount} stack packs passed validation.` : packValidation.errors.join("; ")
  });

  const foundationPackValidation = validateFoundationPacks();
  checks.push({
    name: "foundation pack validation",
    status: foundationPackValidation.valid ? "pass" : "fail",
    summary: foundationPackValidation.valid ? `${foundationPackValidation.packCount} foundation packs passed validation.` : foundationPackValidation.errors.join("; ")
  });

  const policyBundleValidation = validatePolicyBundles();
  checks.push({
    name: "policy bundle validation",
    status: policyBundleValidation.valid ? "pass" : "fail",
    summary: policyBundleValidation.valid ? `${policyBundleValidation.bundleCount} local policy bundles passed validation.` : policyBundleValidation.errors.join("; ")
  });

  const contract = generateContract(ARCHITECT_MCP_BRIEF, ["mcp-server"]);
  const contractValidation = validateArchitectureContract(contract);
  checks.push({
    name: "contract validation",
    status: contractValidation.valid ? "pass" : "fail",
    summary: contractValidation.valid ? `Contract ${contract.contractVersion} is compatible.` : contractValidation.errors.join("; ")
  });

  const artifactValidation = validateRepoArtifacts(generateRepoArtifacts(contract));
  checks.push({
    name: "artifact validation",
    status: artifactValidation.valid ? "pass" : "fail",
    summary: artifactValidation.valid ? "Generated repo artifacts contain required guardrail sections." : artifactValidation.errors.join("; ")
  });

  const selfReview = await runArchitectSelfReview();
  checks.push({
    name: "self review",
    status: selfReview.review.gate.status === "pass" ? "pass" : "fail",
    summary: `Self-review gate ${selfReview.review.gate.status}; score ${selfReview.review.score}; findings ${selfReview.review.findings}.`
  });

  const hostedPolicy = classifyToolPolicy(registeredArchitectureToolNames(false));
  checks.push({
    name: "hosted safety",
    status: hostedPolicy.tools.some((tool) => tool.policy === "local-only") ? "fail" : "pass",
    summary: hostedPolicy.tools.some((tool) => tool.policy === "local-only")
      ? "Hosted tool registry includes local-only tools."
      : `${hostedPolicy.tools.length} hosted-mode tools are classified without local-only filesystem access.`
  });

  const localPolicy = classifyToolPolicy(registeredArchitectureToolNames(true));
  checks.push({
    name: "tool schema policy",
    status: localPolicy.tools.length === registeredArchitectureToolNames(true).length ? "pass" : "fail",
    summary: `${localPolicy.tools.length}/${registeredArchitectureToolNames(true).length} registered tools have hosted policy classification; local-only tools: ${localPolicy.summary.localOnly}.`
  });

  const stagedEval = runV5V9EvalHarness("v9");
  checks.push({
    name: "advanced staged evals",
    status: stagedEval.status === "pass" ? "pass" : "fail",
    summary: `${stagedEval.summary.passed}/${stagedEval.summary.total} advanced maturity evals passed.`
  });

  const v10Eval = runV10EvalHarness();
  checks.push({
    name: "productization boundary evals",
    status: v10Eval.status === "pass" ? "pass" : "fail",
    summary: `${v10Eval.summary.passed}/${v10Eval.summary.total} productization boundary evals passed.`
  });

  const repoQualityEval = runRepoQualityEvalScenarios();
  checks.push({
    name: "repo quality evals",
    status: repoQualityEval.status === "pass" ? "pass" : "fail",
    summary: `${repoQualityEval.summary.passed}/${repoQualityEval.summary.total} repo quality eval scenarios passed.`
  });

  const scriptCheck = checkPackageScripts();
  checks.push({
    name: "package script hygiene",
    status: scriptCheck.missing.length === 0 ? "pass" : "fail",
    summary: scriptCheck.missing.length === 0 ? "Required verification scripts and clean-checkout readiness bootstraps are present." : `Missing package scripts: ${scriptCheck.missing.join(", ")}.`
  });

  return {
    ready: checks.every((check) => check.status !== "fail"),
    checks
  };
}

function checkPackageScripts(): { missing: string[] } {
  try {
    const packageJson = JSON.parse(readFileSync(resolve(process.cwd(), "package.json"), "utf8")) as { scripts?: Record<string, string> };
    const scripts = packageJson.scripts ?? {};
    return {
      missing: requiredScripts().filter((script) => !scripts[script])
    };
  } catch {
    return {
      missing: requiredScripts()
    };
  }
}

function requiredScripts(): string[] {
  return ["typecheck", "test", "build", "audit", "docs:get", "docs:build", "docs:preview", "pack:dry-run", "precheck:v3", "check:v3", "precheck:v5", "check:v5", "precheck:v6", "check:v6", "precheck:v7", "check:v7", "precheck:v8", "check:v8", "precheck:v9", "check:v9", "precheck:v10", "check:v10", "release:check"];
}
