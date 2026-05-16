import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";

describe("supply-chain and release hygiene", () => {
  it("ships license, security policy, env example, and portable MCP setup docs", () => {
    const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
      main: string;
      types: string;
      exports: Record<string, unknown>;
      files: string[];
    };

    assert.equal(existsSync("LICENSE"), true);
    assert.equal(existsSync("SECURITY.md"), true);
    assert.equal(existsSync(".env.example"), true);
    assert.equal(packageJson.main, "./dist/library.js");
    assert.equal(packageJson.types, "./dist/library.d.ts");
    assert.equal(Object.hasOwn(packageJson.exports, "./http"), false);
    assert.equal(packageJson.files.includes("docs"), false);
    assert.equal(packageJson.files.includes("docs/**/*.md"), true);
    assert.equal(packageJson.files.includes("mcp-catalog"), true);
    assert.match(readFileSync(".gitignore", "utf8"), /!\.env\.example/);
    assert.doesNotMatch(readFileSync("README.md", "utf8"), /\/Users\/tonycordner/);
    assert.match(readFileSync("README.md", "utf8"), /npx/);
  });

  it("pins GitHub Actions to full commit SHAs and runs real checks", () => {
    const workflow = readFileSync(".github/workflows/ci.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    for (const command of ["npm run secret:scan", "npm run typecheck", "npm test", "npm run build", "npm audit", "npm run pack:dry-run"]) {
      assert.match(workflow, new RegExp(command.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
    }
  });

  it("pins the GitHub Pages workflow and builds VitePress docs", () => {
    const workflow = readFileSync(".github/workflows/pages.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /npm run docs:build/);
    assert.match(workflow, /pages:\s+write/);
    assert.match(workflow, /id-token:\s+write/);
    assert.match(workflow, /docs\/\.vitepress\/dist/);
  });

  it("publishes npm only after the clean release gate and tarball smoke", () => {
    const workflow = readFileSync(".github/workflows/npm-publish.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /types:\s*\n\s+- published/);
    assert.match(workflow, /npm run release:check/);
    assert.match(workflow, /npm pack --json/);
    assert.match(workflow, /npm install --prefix "\$smoke_dir"/);
    assert.match(workflow, /npm publish --access public --provenance/);
    assert.match(workflow, /NODE_AUTH_TOKEN: \$\{\{ secrets\.NPM_TOKEN \}\}/);
  });

  it("configures Dependabot for npm and GitHub Actions", () => {
    const config = readFileSync(".github/dependabot.yml", "utf8");

    assert.match(config, /package-ecosystem:\s+npm/);
    assert.match(config, /package-ecosystem:\s+github-actions/);
  });
});
