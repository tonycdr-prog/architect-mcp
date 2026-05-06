import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { scanMcpConfigFiles } from "../src/domain/mcpConfigScanner.js";

describe("scanMcpConfigFiles", () => {
  it("scans repo-local .mcp.json files", () => {
    const root = mkdtempSync(join(tmpdir(), "architect-mcp-config-"));
    writeFileSync(join(root, ".mcp.json"), JSON.stringify({
      mcpServers: {
        risky: {
          command: "npx",
          args: ["-y", "some-mcp@latest"]
        }
      }
    }), "utf8");

    const result = scanMcpConfigFiles({ rootPath: root });
    const reviewed = result.results.find((entry) => entry.path.endsWith(".mcp.json"));

    assert.equal(result.scanned, 1);
    assert.equal(reviewed?.status, "reviewed");
    assert.equal(reviewed?.status === "reviewed" && reviewed.review.status, "warn");
  });

  it("reports invalid config parse errors without throwing", () => {
    const root = mkdtempSync(join(tmpdir(), "architect-mcp-config-"));
    writeFileSync(join(root, ".mcp.json"), "{ invalid", "utf8");

    const result = scanMcpConfigFiles({ rootPath: root });

    assert.equal(result.scanned, 1);
    assert.equal(result.results.some((entry) => entry.status === "error"), true);
  });
});
