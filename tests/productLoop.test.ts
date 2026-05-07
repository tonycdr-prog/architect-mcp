import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createBaselineFromFindings } from "../src/domain/baseline.js";
import { generateContract } from "../src/domain/contract.js";
import { generateRepoArtifacts } from "../src/domain/artifacts.js";
import { scoreAgentInstructions } from "../src/domain/artifactQuality.js";
import { validateRepoArtifacts } from "../src/domain/artifactValidation.js";
import { grillMe } from "../src/domain/intake.js";
import { validateArchitectureContract } from "../src/domain/contractValidation.js";
import { validateFoundationPacks } from "../src/domain/foundationPacks.js";
import { createReviewReport } from "../src/domain/reviewReport.js";
import { reviewFileSummaries } from "../src/domain/reviewer.js";
import { classifyReviewLifecycle } from "../src/domain/reviewLifecycle.js";
import { validateStackPacks } from "../src/domain/stackPacks.js";
import {
  cleanMcpServerFixture,
  existingLayoutFixture,
  messyReactFixture,
  migrationBaselineFixture
} from "./fixtures/repos.js";
import {
  expoGeneratedBadFixture,
  expoGeneratedGoodFixture,
  paidAuthSupabaseBadFixture,
  paidAuthSupabaseGoodFixture
} from "./fixtures/v3Repos.js";

describe("local product loop", () => {
  it("goes from grill-me to contract to passing review for a clean MCP fixture", () => {
    const grilled = grillMe(cleanMcpServerFixture.brief, {
      stackPackIds: cleanMcpServerFixture.stackPackIds,
      includeContract: true,
      includeArtifacts: true
    });

    assert.equal(grilled.ready, true);
    assert.equal(grilled.specCompleteness.score >= 75, true);
    assert.deepEqual(grilled.selectedStackPacks, ["mcp-server"]);
    assert.equal(grilled.archetype, "ai-workflow-tool");
    assert.equal(grilled.foundationPacks.map((pack) => pack.id).includes("agent-harness"), true);
    assert.equal(grilled.buildPlan.slices.some((slice) => slice.id === "agent-harness"), true);
    assert.equal(grilled.buildPlan.slices.every((slice) => slice.inputs.length > 0 && slice.outputs.length > 0), true);
    assert.equal(grilled.artifacts?.some((artifact) => artifact.path === "AGENTS.md"), true);
    assert.equal(grilled.artifacts?.some((artifact) => artifact.path === ".architectignore"), true);
    assert.equal(grilled.artifacts?.some((artifact) => artifact.path === "docs/build-plan.md"), true);
    assert.match(grilled.markdown ?? "", /Contract version: 0\.2\.0/);
    assert.match(grilled.markdown ?? "", /Review Gate/);
    assert.match(grilled.markdown ?? "", /Baseline Lifecycle/);
    assert.match(grilled.markdown ?? "", /Pre-Coding Checklist/);
    assert.match(grilled.markdown ?? "", /App Generation Guardrails/);
    assert.match(grilled.markdown ?? "", /Agent Harness Setup/);
    assert.match(grilled.artifacts?.find((artifact) => artifact.path === "AGENTS.md")?.content ?? "", /Do Not Create/);

    const contract = grilled.contract ?? generateContract(cleanMcpServerFixture.brief, cleanMcpServerFixture.stackPackIds);
    assert.equal(contract.contractVersion, "0.2.0");
    assert.equal(contract.generatedBy.tool, "architect-mcp");
    assert.equal(validateArchitectureContract(contract).valid, true);
    const findings = reviewFileSummaries(cleanMcpServerFixture.files, contract, 300, cleanMcpServerFixture.directories);
    const report = createReviewReport(findings, { mode: "ci" });

    assert.equal(report.gate.status, "pass");
    assert.equal(report.summary.errors, 0);
    assert.equal(report.summary.warnings, 0);
  });

  it("pressure-tests and fails a messy frontend/database fixture", () => {
    const grilled = grillMe(messyReactFixture.brief, {
      includeContract: true
    });

    assert.equal(grilled.ready, false);
    assert.equal(grilled.blockers.some((blocker) => blocker.includes("server boundary")), true);
    assert.equal(grilled.challenges.some((challenge) => challenge.field === "stack.backend"), true);
    assert.equal(grilled.challenges.some((challenge) => challenge.field === "dataEntities"), true);
    assert.equal(grilled.buildPlan.slices.some((slice) => slice.id === "database-boundary"), true);

    const contract = grilled.contract ?? generateContract(messyReactFixture.brief);
    const findings = reviewFileSummaries(messyReactFixture.files, contract, 300, messyReactFixture.directories);
    const report = createReviewReport(findings, { mode: "ci" });

    assert.equal(report.gate.status, "fail");
    assert.equal(findings.some((finding) => finding.code === "ARCH002_UI_DB_ACCESS"), true);
    assert.equal(findings.some((finding) => finding.code === "ARCH009_SERVER_ENV_IN_UI"), true);
  });

  it("keeps existing non-src repo layouts mapped into generated contracts", () => {
    const grilled = grillMe(existingLayoutFixture.brief, {
      includeContract: true
    });

    const contract = grilled.contract ?? generateContract(existingLayoutFixture.brief);
    assert.match(grilled.markdown ?? "", /Repo Layout/);
    assert.match(grilled.markdown ?? "", /adapted to the target repo layout/);
    const paths = contract.directories.map((directory) => directory.path);
    const findings = reviewFileSummaries(existingLayoutFixture.files, contract, 300, existingLayoutFixture.directories);
    const report = createReviewReport(findings, { mode: "migration" });

    assert.equal(paths.includes("client"), true);
    assert.equal(paths.includes("server/routes"), true);
    assert.equal(paths.includes("src/features"), false);
    assert.equal(report.summary.errors, 0);
  });

  it("classifies baseline, accepted, resolved, and new findings in a migration repo", () => {
    const contract = generateContract(migrationBaselineFixture.brief);
    const originalFindings = reviewFileSummaries(migrationBaselineFixture.files, contract, 300, migrationBaselineFixture.directories);
    const baseline = createBaselineFromFindings(originalFindings);
    baseline.findings[0] = {
      ...baseline.findings[0],
      status: "accepted",
      reason: "Large legacy dashboard is accepted until the planned split."
    };
    baseline.findings.push({
      code: "ARCH002_UI_DB_ACCESS",
      path: "src/features/resolved/OldDbPanel.tsx",
      message: "Resolved historical finding."
    });

    const currentFindings = [
      ...originalFindings,
      {
        code: "ARCH009_SERVER_ENV_IN_UI" as const,
        confidence: "high" as const,
        severity: "error" as const,
        path: "src/features/new/NewPanel.tsx",
        message: "UI file reads server-only environment variables.",
        recommendation: "Move secret access into server-only code."
      }
    ];
    const lifecycle = classifyReviewLifecycle(currentFindings, baseline);

    assert.equal(lifecycle.acceptedFindings.length, 1);
    assert.equal(lifecycle.baselineFindings.length, originalFindings.length - 1);
    assert.equal(lifecycle.newFindings.length, 1);
    assert.equal(lifecycle.resolvedFindings.length, 1);
  });

  it("requires quality metadata on every stack pack", () => {
    const validation = validateStackPacks();

    assert.equal(validation.valid, true);
    assert.equal(validation.packCount >= 6, true);
  });

  it("requires quality metadata on every foundation pack", () => {
    const validation = validateFoundationPacks();

    assert.equal(validation.valid, true);
    assert.equal(validation.packCount, 5);
  });

  it("applies a /grill-me answer and returns the updated brief", () => {
    const grilled = grillMe({
      idea: "A small admin tool"
    }, {
      includeContract: false,
      answer: {
        field: "users",
        value: "Operations admins who need to triage customer records."
      }
    });

    assert.equal(grilled.updatedBrief.users, "Operations admins who need to triage customer records.");
    assert.equal(grilled.coveredFields.includes("users"), true);
    assert.equal(grilled.specCompleteness.checks.some((check) => check.id === "users" && check.status === "pass"), true);
  });

  it("rejects contracts that omit foundation guardrails", () => {
    const contract = generateContract(cleanMcpServerFixture.brief, cleanMcpServerFixture.stackPackIds);
    const validation = validateArchitectureContract({
      ...contract,
      foundationPacks: contract.foundationPacks.filter((pack) => pack.id !== "agent-harness")
    });

    assert.equal(validation.valid, false);
    assert.equal(validation.errors.some((error) => error.includes("agent-harness")), true);
  });

  it("warns on duplicate contract entries", () => {
    const contract = generateContract(cleanMcpServerFixture.brief, cleanMcpServerFixture.stackPackIds);
    contract.directories.push(contract.directories[0]!);
    contract.fileRules.push(contract.fileRules[0]!);
    contract.moduleBoundaries.push(contract.moduleBoundaries[0]!);

    const validation = validateArchitectureContract(contract);

    assert.equal(validation.valid, true);
    assert.equal(validation.warnings.some((warning) => warning.includes("Duplicate directory path")), true);
    assert.equal(validation.warnings.some((warning) => warning.includes("Duplicate file rule")), true);
    assert.equal(validation.warnings.some((warning) => warning.includes("Duplicate module boundary")), true);
  });

  it("covers Expo generated app fixtures", () => {
    const badContract = generateContract(expoGeneratedBadFixture.brief, expoGeneratedBadFixture.stackPackIds);
    const badFindings = reviewFileSummaries(expoGeneratedBadFixture.files, badContract, 300, expoGeneratedBadFixture.directories);
    assert.equal(createReviewReport(badFindings, { mode: "ci" }).gate.status, "warn");
    assert.equal(badFindings.some((finding) => finding.path === "src/screens/InspectionScreen.tsx" && finding.code === "ARCH001_OVERSIZED_FILE"), true);

    const goodContract = generateContract(expoGeneratedGoodFixture.brief, expoGeneratedGoodFixture.stackPackIds);
    const goodFindings = reviewFileSummaries(expoGeneratedGoodFixture.files, goodContract, 300, expoGeneratedGoodFixture.directories);
    assert.equal(createReviewReport(goodFindings, { mode: "ci" }).gate.status, "pass");
  });

  it("covers paid Auth0 Stripe Supabase fixtures", () => {
    const badContract = generateContract(paidAuthSupabaseBadFixture.brief, paidAuthSupabaseBadFixture.stackPackIds);
    const badFindings = reviewFileSummaries(paidAuthSupabaseBadFixture.files, badContract, 300, paidAuthSupabaseBadFixture.directories);
    const badMessages = badFindings.map((finding) => finding.message).join("\n");
    assert.equal(createReviewReport(badFindings, { mode: "ci" }).gate.status, "fail");
    assert.match(badMessages, /Auth Server Boundary/);
    assert.match(badMessages, /Server\/client Supabase split/);
    assert.match(badMessages, /Stripe Payment Server Boundary/);
    assert.match(badMessages, /AI Tool Safety Boundary/);

    const goodContract = generateContract(paidAuthSupabaseGoodFixture.brief, paidAuthSupabaseGoodFixture.stackPackIds);
    const goodFindings = reviewFileSummaries(paidAuthSupabaseGoodFixture.files, goodContract, 300, paidAuthSupabaseGoodFixture.directories);
    assert.equal(createReviewReport(goodFindings, { mode: "ci" }).gate.status, "pass");
  });

  it("rejects placeholder repo artifacts and self-scores generated AGENTS.md", () => {
    const marker = "Stack Packs Pre-Coding Checklist App Generation Guardrails Agent Harness Setup Review Gate Baseline Lifecycle Architecture review";
    const placeholderArtifacts = ["AGENTS.md", "docs/architecture-contract.md", ".cursor/rules/architecture.mdc"].map((path) => ({
      path,
      description: "placeholder",
      content: marker
    }));
    placeholderArtifacts.push({ path: ".architectignore", description: "ignore", content: "node_modules/**" });
    placeholderArtifacts.push({ path: "docs/build-plan.md", description: "plan", content: "# Build Plan" });

    assert.equal(validateRepoArtifacts(placeholderArtifacts).valid, false);

    const contract = generateContract(cleanMcpServerFixture.brief, cleanMcpServerFixture.stackPackIds);
    const agentsMd = generateRepoArtifacts(contract, cleanMcpServerFixture.brief).find((artifact) => artifact.path === "AGENTS.md")?.content ?? "";
    assert.equal(scoreAgentInstructions(agentsMd).status, "pass");
  });
});
