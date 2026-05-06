import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

describe("V4 and V5 scope boundaries", () => {
  it("keeps hosted, database/storage, and GitHub adapter work out of V4", () => {
    const v4 = readFileSync("docs/v4-scope.md", "utf8");
    const v4Audit = readFileSync("docs/v4-scope-audit.md", "utf8");

    assert.match(v4, /must not include hosted service work, database\/storage work, GitHub\/PR adapters/);
    assert.match(v4, /Change Review/);
    assert.doesNotMatch(v4, /^### PR Review$/m);
    assert.doesNotMatch(v4, /user-owned GitHub\/PR adapters/);
    assert.doesNotMatch(v4, /slow database query patterns/);
    assert.doesNotMatch(v4, /^Migrations:$/m);
    assert.match(v4Audit, /Deferred Until Backend\/Productization Phase/);
    assert.doesNotMatch(v4, /move to V5/);
  });

  it("defines V5 as local-first product depth without productization layers", () => {
    const v5 = readFileSync("docs/v5-scope.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(v5, /V5 is the next local-first product depth after V4/);
    assert.match(v5, /Standards Orchestration/);
    assert.match(v5, /Explainable Findings/);
    assert.match(v5, /Policy Simulation/);
    assert.match(v5, /Repo Profile Intelligence/);
    assert.match(v5, /Hosted service or hosted API implementation/);
    assert.match(v5, /Database\/storage, including schema, persistence, migrations, review history, or memory storage/);
    assert.match(v5, /GitHub repository, branch, PR, issue, app, OAuth, or comment workflow/);
    assert.match(llms, /docs\/v5-scope\.md/);
  });

  it("defines V6 as local governance maturity without productization layers", () => {
    const v6 = readFileSync("docs/v6-scope.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(v6, /V6 is the next local-first maturity layer after V5/);
    assert.match(v6, /Policy Composition/);
    assert.match(v6, /Standards Lifecycle/);
    assert.match(v6, /Multi-Turn Agent Continuity/);
    assert.match(v6, /Review Explainability At Scale/);
    assert.match(v6, /Local Report Artifacts/);
    assert.match(v6, /Regression Intelligence/);
    assert.match(v6, /Cross-Repo Pattern Portability/);
    assert.match(v6, /Hosted service or hosted API implementation/);
    assert.match(v6, /Database\/storage, including schema, persistence, migrations, review history, or memory storage/);
    assert.match(v6, /GitHub repository, branch, PR, issue, app, OAuth, or comment workflow/);
    assert.match(llms, /docs\/v6-scope\.md/);
  });

  it("defines V7 as local strategic intelligence without productization layers", () => {
    const v7 = readFileSync("docs/v7-scope.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(v7, /V7 is the next local-first maturity layer after V6/);
    assert.match(v7, /Architecture Strategy Maps/);
    assert.match(v7, /Standards Negotiation/);
    assert.match(v7, /Local What-If Planning/);
    assert.match(v7, /Agent Behavior Diagnostics/);
    assert.match(v7, /Rule Authoring Assistance/);
    assert.match(v7, /Quality Trend Snapshots/);
    assert.match(v7, /Human-Readable Governance Packs/);
    assert.match(v7, /Hosted service or hosted API implementation/);
    assert.match(v7, /Database\/storage, including schema, persistence, migrations, review history, or memory storage/);
    assert.match(v7, /GitHub repository, branch, PR, issue, app, OAuth, or comment workflow/);
    assert.match(llms, /docs\/v7-scope\.md/);
  });

  it("defines V8 as local governance automation without productization layers", () => {
    const v8 = readFileSync("docs/v8-scope.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(v8, /V8 is the next local-first maturity layer after V7/);
    assert.match(v8, /Standards Refactoring/);
    assert.match(v8, /Policy Minimization/);
    assert.match(v8, /Local Review Playbooks/);
    assert.match(v8, /Agent Collaboration Protocols/);
    assert.match(v8, /Failure-Mode Drills/);
    assert.match(v8, /Rule Impact Calibration/);
    assert.match(v8, /Documentation Intelligence/);
    assert.match(v8, /Hosted service or hosted API implementation/);
    assert.match(v8, /Database\/storage, including schema, persistence, migrations, review history, or memory storage/);
    assert.match(v8, /GitHub repository, branch, PR, issue, app, OAuth, or comment workflow/);
    assert.match(llms, /docs\/v8-scope\.md/);
  });

  it("audits V5 through V8 and defines V9 as local operating-model readiness", () => {
    const audit = readFileSync("docs/v5-v8-scope-audit.md", "utf8");
    const v9 = readFileSync("docs/v9-scope.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(audit, /The main gap is not missing productization/);
    assert.match(audit, /local operating model/);
    assert.match(audit, /Older V4 Wording Made V5 Sound Like The Productization Gate/);
    assert.match(audit, /Source Provenance Needs To Flow Through Later Workflow Layers/);
    assert.match(v9, /V9 is the next local-first operating-model layer after V8/);
    assert.match(v9, /Local Orchestration Recipes/);
    assert.match(v9, /Scenario Acceptance Profiles/);
    assert.match(v9, /Result Normalization/);
    assert.match(v9, /Context-Budget Governance/);
    assert.match(v9, /Evidence Routing/);
    assert.match(v9, /Source Provenance Continuity/);
    assert.match(v9, /Advisory Skill-Pattern Routing/);
    assert.match(v9, /Artifact-Quality Routing/);
    assert.match(v9, /Local Dry-Run Plans/);
    assert.match(v9, /Tool-Loop Quality Gates/);
    assert.match(v9, /Hosted service or hosted API implementation/);
    assert.match(v9, /Database\/storage, including schema, persistence, migrations, review history, or memory storage/);
    assert.match(v9, /GitHub repository, branch, PR, issue, app, OAuth, or comment workflow/);
    assert.match(llms, /docs\/v5-v8-scope-audit\.md/);
    assert.match(llms, /docs\/v9-scope\.md/);
  });

  it("keeps the V10 concept doc as the productization starting point", () => {
    const v10 = readFileSync("docs/v10-productization-concepts.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(v10, /V10 is a concept exploration/);
    assert.match(v10, /Accounts and identity/);
    assert.match(v10, /Organizations and teams/);
    assert.match(v10, /Billing and plan boundaries/);
    assert.match(v10, /Product dashboards/);
    assert.match(v10, /Remote policy management/);
    assert.match(llms, /docs\/v10-productization-concepts\.md/);
    assert.match(llms, /docs\/v10-productization-implementation\.md/);
    assert.match(llms, /docs\/v10-architecture-contract\.md/);
  });
});
