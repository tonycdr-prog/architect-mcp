import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { classifyToolPolicy } from "../src/domain/toolPolicy.js";
import { createStackPackCoverageMatrix, scoreStackPacks } from "../src/domain/packReports.js";

describe("hosted policy and pack reports", () => {
  it("classifies hosted-safe, local-only, and future-adapter tools", () => {
    const report = classifyToolPolicy(["review_repo_structure", "scan_mcp_config_files", "extract_harness_memory", "not_a_real_tool"]);

    assert.equal(report.tools.find((tool) => tool.name === "review_repo_structure")?.policy, "hosted-safe");
    assert.equal(report.tools.find((tool) => tool.name === "scan_mcp_config_files")?.policy, "local-only");
    assert.equal(report.tools.find((tool) => tool.name === "extract_harness_memory")?.policy, "future-adapter");
    assert.equal(report.tools.find((tool) => tool.name === "not_a_real_tool")?.policy, "unknown");
    assert.equal(report.warnings.length, 1);
  });

  it("scores stack packs and exposes coverage matrix rows", () => {
    const scores = scoreStackPacks();
    const matrix = createStackPackCoverageMatrix();

    assert.equal(scores.packs.some((pack) => pack.id === "stripe" && pack.status === "pass"), true);
    assert.equal(matrix.rows.some((row) => row.id === "hono" && row.hasIngestedLlms && row.hasExecutableDetector), true);
  });
});
