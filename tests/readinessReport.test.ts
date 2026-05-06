import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createMcpReadinessReport } from "../src/domain/readinessReport.js";

describe("V3 readiness release gates", () => {
  it("requires release-check scripts and hosted API docs", async () => {
    const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
      scripts: Record<string, string>;
      files: string[];
    };
    const readiness = await createMcpReadinessReport();
    const scriptCheck = readiness.checks.find((check) => check.name === "package script hygiene");

    assert.equal(readiness.ready, true);
    assert.equal(scriptCheck?.status, "pass");
    assert.equal(packageJson.scripts["check:v3"], "tsx scripts/checkV3Readiness.ts");
    assert.equal(packageJson.scripts["release:check"], "npm run check:v3");
    assert.equal(packageJson.files.includes("scripts"), true);
    assert.match(readFileSync("docs/hosted-api-shape.md", "utf8"), /POST \/v1\/reviews\/session/);
    assert.match(readFileSync("README.md", "utf8"), /npm run check:v3/);
    assert.match(readFileSync("llms.txt", "utf8"), /docs\/hosted-api-shape\.md/);
  });
});
