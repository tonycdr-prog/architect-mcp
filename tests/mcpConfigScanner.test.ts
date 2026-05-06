import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
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

  it("parses JSONC comments without corrupting URLs or comment-like strings", () => {
    const root = mkdtempSync(join(tmpdir(), "architect-mcp-config-"));
    mkdirSync(join(root, ".cursor"));
    writeFileSync(join(root, ".cursor", "mcp.jsonc"), [
      "{",
      "  // local Cursor MCP config",
      "  \"mcpServers\": {",
      "    \"docs\": {",
      "      \"command\": \"node\",",
      "      \"args\": [\"https://example.com/path//kept\", \"literal /* kept */ text\"]",
      "    }",
      "  }",
      "}"
    ].join("\n"), "utf8");

    const result = scanMcpConfigFiles({ rootPath: root, approvedServers: ["docs"] });
    const reviewed = result.results.find((entry) => entry.path.endsWith("mcp.jsonc"));

    assert.equal(reviewed?.status, "reviewed");
    assert.equal(reviewed?.status === "reviewed" && reviewed.review.findings.some((finding) => /example\.com/.test(finding.message)), false);
  });
});
