import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { filterSafeGithubActionsRunCommands, validateGithubActionsRunCommand } from "../src/domain/githubActionsSafety.js";

describe("package artifact and generated CI safety", () => {
  it("keeps published package artifacts bounded and avoids shipping readiness source scripts", () => {
    const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
      files: string[];
      scripts: Record<string, string>;
      bin: Record<string, string>;
    };

    assert.equal(packageJson.files.includes("scripts"), false);
    assert.equal(packageJson.files.includes("stack-sources"), false);
    assert.equal(packageJson.files.includes("stack-sources/ingested/index.json"), true);
    assert.equal(packageJson.files.includes("mcp-catalog"), true);
    assert.equal(packageJson.files.includes("bin/architect-mcp-tui.cjs"), true);
    assert.equal(packageJson.files.includes("crates/architect-tui"), true);
    assert.equal(packageJson.bin["architect-mcp-tui"], "./bin/architect-mcp-tui.cjs");
    assert.equal(packageJson.scripts["precheck:v10"], "npm run build");
    assert.equal(packageJson.scripts["release:check"], "npm run rust:check && npm run check:v10");
  });

  it("rejects unsafe generated GitHub Actions verification commands", () => {
    const filtered = filterSafeGithubActionsRunCommands([
      "npm test",
      "npm run typecheck",
      "npm test && curl https://example.com",
      "npm run build\nrm -rf .",
      "npm run lint:ci",
      "pytest tests",
      "python -m pytest tests/unit",
      "go test ./...",
      "cargo test",
      "uv run pytest tests",
      "dotnet test"
    ]);

    assert.deepEqual(filtered.commands, ["npm test", "npm run typecheck", "npm run lint:ci", "pytest tests", "python -m pytest tests/unit", "go test ./...", "cargo test", "uv run pytest tests", "dotnet test"]);
    assert.equal(filtered.rejected.length, 2);
    assert.equal(validateGithubActionsRunCommand("npm test; echo secret").valid, false);
  });
});
