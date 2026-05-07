import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";

describe("supply-chain and release hygiene", () => {
  it("ships license, security policy, env example, and portable MCP setup docs", () => {
    assert.equal(existsSync("LICENSE"), true);
    assert.equal(existsSync("SECURITY.md"), true);
    assert.equal(existsSync(".env.example"), true);
    assert.match(readFileSync(".gitignore", "utf8"), /!\.env\.example/);
    assert.doesNotMatch(readFileSync("README.md", "utf8"), /\/Users\/tonycordner/);
    assert.match(readFileSync("README.md", "utf8"), /npx/);
  });

  it("pins GitHub Actions to full commit SHAs and runs real checks", () => {
    const workflow = readFileSync(".github/workflows/ci.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}\s*$/.test(line.trim())), true);
    for (const command of ["npm run secret:scan", "npm run typecheck", "npm test", "npm run build", "npm audit", "npm run pack:dry-run"]) {
      assert.match(workflow, new RegExp(command.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
    }
  });

  it("configures Dependabot for npm and GitHub Actions", () => {
    const config = readFileSync(".github/dependabot.yml", "utf8");

    assert.match(config, /package-ecosystem:\s+npm/);
    assert.match(config, /package-ecosystem:\s+github-actions/);
  });
});

