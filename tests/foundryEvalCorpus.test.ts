import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { foundryEvalCorpusCases, runFoundryEvalCorpus } from "../src/domain/foundryEvalCorpus.js";

describe("Foundry eval corpus", () => {
  it("runs offline fixtures that cover all Foundry decision routes", () => {
    const report = runFoundryEvalCorpus();

    assert.equal(report.status, "pass");
    assert.equal(report.summary.offlineNetworkRequired, false);
    assert.equal(report.summary.serverWritesPerformed, 0);
    assert.equal(report.requirements.noMutation, true);
    assert.equal(report.requirements.publicSafety, true);
    for (const route of ["pr_preview", "architect_issue", "exception", "no_op", "ask_human"] as const) {
      assert.equal(report.requirements.routeCoverage[route], true, `${route} was not covered`);
      assert.equal((report.summary.byRoute[route] ?? 0) > 0, true, `${route} count missing`);
    }
  });

  it("covers the Foundry signal-quality regression issue set", () => {
    const report = runFoundryEvalCorpus();

    for (const issue of [310, 311, 312, 313, 314, 315, 316, 317, 318, 319]) {
      assert.equal(report.requirements.regressionIssues[String(issue)], true, `#${issue} was not covered`);
    }
    assert.deepEqual(report.summary.suppressionCategories.sort(), [
      "conventional_entrypoint",
      "docs_example",
      "generated_file"
    ]);
  });

  it("returns only public-safe summarized evidence", () => {
    const report = runFoundryEvalCorpus();
    const serialized = JSON.stringify(report);

    assert.equal(report.publicSafety.passed, true);
    assert.doesNotMatch(serialized, /\/Users\//);
    assert.doesNotMatch(serialized, /\/home\/|\/tmp\/|\/var\/folders\/|\/Volumes\//);
    assert.doesNotMatch(serialized, /ghp_secretcorp1234567890/);
    assert.doesNotMatch(serialized, /sk-secretcorpus123456/);
    assert.doesNotMatch(serialized, /RAW_PRIVATE_PAYLOAD/);
    assert.doesNotMatch(serialized, /private stack trace/);
    assert.doesNotMatch(serialized, /secret internal path/);
    assert.equal(report.results.find((result) => result.id === "tiny-package-noop-and-human")?.publicSafety.passed, true);
  });

  it("documents optional read-only live smoke without making it part of offline pass", () => {
    const report = runFoundryEvalCorpus();

    assert.equal(report.requirements.optionalLiveSmoke, true);
    assert.equal(report.liveSmoke.mode, "optional_read_only");
    assert.equal(report.liveSmoke.networkRequired, true);
    assert.match(report.liveSmoke.command, /foundry-audit --repo-path/);
    assert.equal(report.liveSmoke.noMutationChecks.some((check) => /git status --short/.test(check)), true);
  });

  it("supports running a focused subset by case id", () => {
    const [first] = foundryEvalCorpusCases();
    const report = runFoundryEvalCorpus({ caseIds: [first.id] });

    assert.equal(report.summary.totalCases, 1);
    assert.equal(report.results[0].id, first.id);
    assert.equal(report.results[0].status, "pass");
  });
});
