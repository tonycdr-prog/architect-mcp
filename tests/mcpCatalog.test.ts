import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import {
  applyMcpInstallPlan,
  createMcpInstallPlan,
  listMcpServerCatalog,
  recommendMcpServers,
  reviewMcpInstallPlan,
  validateMcpServerCatalog
} from "../src/domain/mcpCatalog.js";

describe("mcp catalog", () => {
  it("validates the source-backed catalog and lists known servers", () => {
    const validation = validateMcpServerCatalog();
    const catalog = listMcpServerCatalog({ query: "database" });

    assert.equal(validation.valid, true);
    assert.equal(catalog.servers.some((server) => server.id === "supabase"), true);
    assert.equal(catalog.sources.length > 0, true);
  });

  it("asks for a provider instead of defaulting generic database needs to Supabase", () => {
    const recommendation = recommendMcpServers({
      request: "Build a customer portal with a database and auth."
    });

    assert.equal(recommendation.status, "needs-clarification");
    assert.equal(recommendation.recommendations.some((server) => server.serverId === "supabase"), false);
    assert.match(recommendation.questions.join(" "), /database or backend provider/);
  });

  it("recommends Supabase only from explicit provider context", () => {
    const recommendation = recommendMcpServers({
      request: "Build this with Supabase auth and Postgres."
    });

    assert.equal(recommendation.recommendations.some((server) => server.serverId === "supabase"), true);
  });

  it("requires a confirmed server-side boundary for Stripe recommendations", () => {
    const missingBoundary = recommendMcpServers({
      request: "Use Stripe for checkout."
    });
    const withBoundary = recommendMcpServers({
      request: "Use Stripe for checkout with server-side webhook handling.",
      serverSideBoundaryConfirmed: true
    });

    assert.equal(missingBoundary.recommendations.some((server) => server.serverId === "stripe"), false);
    assert.match(missingBoundary.questions.join(" "), /server-side Stripe boundary/);
    assert.equal(withBoundary.recommendations.some((server) => server.serverId === "stripe"), true);
  });

  it("creates and reviews pinned dry-run install plans", () => {
    const plan = createMcpInstallPlan({ serverId: "playwright", targetClient: "generic-json" });
    const review = reviewMcpInstallPlan({ plan });
    const dryRun = applyMcpInstallPlan({ plan, targetPath: ".mcp.json" });

    assert.equal(plan.status, "dry-run");
    assert.equal(JSON.stringify(plan).includes("@latest"), false);
    assert.equal(review.status, "pass");
    assert.equal(dryRun.status, "dry-run");
    assert.equal(existsSync(".mcp.json"), false);
  });

  it("blocks unknown servers and hosted local writes", () => {
    assert.throws(() => createMcpInstallPlan({ serverId: "not-real" }), /Unknown MCP server/);

    const hostedPlan = createMcpInstallPlan({ serverId: "playwright", targetClient: "generic-json", hostedMode: true });
    const review = reviewMcpInstallPlan({ plan: hostedPlan, writeFiles: true, explicitApproval: true });

    assert.equal(hostedPlan.localOnly, true);
    assert.equal(review.status, "fail");
    assert.equal(review.findings.some((finding) => finding.code === "MCPINSTALL005_HOSTED_WRITE"), true);
  });

  it("writes project-local config only with explicit approval", () => {
    const previousCwd = process.cwd();
    const tempDir = mkdtempSync(join(tmpdir(), "architect-mcp-catalog-"));
    try {
      process.chdir(tempDir);
      const plan = createMcpInstallPlan({ serverId: "memory", targetClient: "generic-json" });
      const blocked = applyMcpInstallPlan({ plan, targetPath: ".mcp.json", writeFiles: true });
      const written = applyMcpInstallPlan({ plan, targetPath: ".mcp.json", writeFiles: true, explicitApproval: true });
      const content = readFileSync(join(tempDir, ".mcp.json"), "utf8");

      assert.equal(blocked.status, "blocked");
      assert.equal(written.status, "written");
      assert.match(content, /@modelcontextprotocol\/server-memory@2026\.1\.26/);
    } finally {
      process.chdir(previousCwd);
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});
