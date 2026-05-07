import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { reviewMcpConfigSecurity } from "../src/domain/mcpSecurity.js";

const configs = JSON.parse(readFileSync("tests/fixtures/v3/mcp-configs.json", "utf8")) as Record<string, unknown>;

describe("reviewMcpConfigSecurity", () => {
  it("passes env-var based pinned MCP configs", () => {
    const review = reviewMcpConfigSecurity({
      config: configs.safe,
      approvedServers: ["example"]
    });

    assert.equal(review.status, "pass");
    assert.equal(review.findings.length, 0);
  });

  it("flags hardcoded secrets and unpinned packages", () => {
    const review = reviewMcpConfigSecurity({ config: configs.riskyLatest });

    assert.equal(review.status, "fail");
    assert.equal(review.findings.some((finding) => finding.code === "MCPSEC001_HARDCODED_SECRET"), true);
    assert.equal(review.findings.some((finding) => finding.code === "MCPSEC003_UNPINNED_DEPENDENCY"), true);
  });

  it("flags shell execution and supports Cursor and Claude Desktop style shapes", () => {
    const shell = reviewMcpConfigSecurity({ config: configs.shellRisk });
    const cursor = reviewMcpConfigSecurity({ config: configs.cursorShape, approvedServers: ["filesystem"] });
    const claude = reviewMcpConfigSecurity({ config: configs.claudeDesktopShape, approvedServers: ["github"] });

    assert.equal(shell.status, "fail");
    assert.equal(shell.findings.some((finding) => finding.code === "MCPSEC002_SHELL_INJECTION" || finding.code === "MCPSEC006_RISKY_COMMAND"), true);
    assert.equal(cursor.status, "pass");
    assert.equal(claude.status, "pass");
  });

  it("warns when servers are not approved", () => {
    const review = reviewMcpConfigSecurity({
      config: configs.safe,
      approvedServers: ["different-server"]
    });

    assert.equal(review.status, "warn");
    assert.equal(review.findings.some((finding) => finding.code === "MCPSEC004_UNAPPROVED_SERVER"), true);
  });

  it("flags semver ranges as unpinned executable MCP dependencies", () => {
    const review = reviewMcpConfigSecurity({
      config: {
        mcpServers: {
          ranged: {
            command: "npx",
            args: ["-y", "@scope/server@^1.2.3", "other-server@~2.0.0"]
          }
        }
      }
    });

    assert.equal(review.status, "warn");
    assert.equal(review.findings.filter((finding) => finding.code === "MCPSEC003_UNPINNED_DEPENDENCY").length, 2);
  });

  it("accepts exact semver with v prefix or build metadata and rejects non-registry specs", () => {
    const review = reviewMcpConfigSecurity({
      config: {
        mcpServers: {
          exact: {
            command: "npx",
            args: ["-y", "@scope/server@v1.2.3", "other-server@1.2.3+build.5", "git+https://example.com/server.git"]
          }
        }
      }
    });

    assert.equal(review.status, "warn");
    assert.equal(review.findings.filter((finding) => finding.code === "MCPSEC003_UNPINNED_DEPENDENCY").length, 1);
  });
});
