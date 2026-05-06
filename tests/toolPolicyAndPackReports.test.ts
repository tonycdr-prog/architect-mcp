import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { classifyToolPolicy } from "../src/domain/toolPolicy.js";
import { createStackPackCoverageMatrix, scoreStackPacks } from "../src/domain/packReports.js";
import { registeredArchitectureToolNames } from "../src/tools/toolRegistry.js";

describe("hosted policy and pack reports", () => {
  it("classifies hosted-safe, local-only, and future-adapter tools", () => {
    const report = classifyToolPolicy(["review_repo_structure", "scan_mcp_config_files", "extract_harness_memory"]);

    assert.equal(report.tools.find((tool) => tool.name === "review_repo_structure")?.policy, "hosted-safe");
    assert.equal(report.tools.find((tool) => tool.name === "scan_mcp_config_files")?.policy, "local-only");
    assert.equal(report.tools.find((tool) => tool.name === "extract_harness_memory")?.policy, "future-adapter");
  });

  it("classifies the shared registered tool list including V10 and repo quality tools", () => {
    const toolNames = registeredArchitectureToolNames(true);
    const originalOrder = [...toolNames];
    const report = classifyToolPolicy(toolNames);

    assert.deepEqual(toolNames, originalOrder);
    assert.equal(report.tools.some((tool) => tool.name === "get_v10_productization_blueprint"), true);
    assert.equal(report.tools.some((tool) => tool.name === "audit_generated_repo_quality"), true);
    assert.equal(report.tools.some((tool) => tool.name === "review_local_workspace" && tool.policy === "local-only"), true);
  });

  it("scores stack packs and exposes coverage matrix rows", () => {
    const scores = scoreStackPacks();
    const matrix = createStackPackCoverageMatrix();

    assert.equal(scores.packs.some((pack) => pack.id === "stripe" && pack.status === "pass"), true);
    assert.equal(matrix.rows.some((row) => row.id === "hono" && row.hasIngestedLlms && row.hasExecutableDetector), true);
  });
});
