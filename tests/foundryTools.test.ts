import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";

describe("Foundry MCP tools", () => {
  it("derives repo constitution from supplied and local evidence", async () => {
    const { client, close } = await connectTestClient();
    const root = mkdtempSync(join(tmpdir(), "architect-mcp-constitution-"));
    try {
      mkdirSync(join(root, ".github", "workflows"), { recursive: true });
      writeFileSync(join(root, "AGENTS.md"), "Use the local work gate.\n", "utf8");
      writeFileSync(join(root, "README.md"), "# Demo\n", "utf8");
      writeFileSync(join(root, "package.json"), JSON.stringify({ name: "demo", version: "1.0.0", scripts: { test: "node --test" } }), "utf8");
      writeFileSync(join(root, ".github", "pull_request_template.md"), "## Summary\n\n## Verification\n- [ ] npm test\n", "utf8");
      writeFileSync(join(root, ".github", "workflows", "ci.yml"), "name: CI\non:\n  pull_request:\n", "utf8");
      writeFileSync(join(root, "index.ts"), "export const value = 1;\n", "utf8");

      const supplied = await callJson(client, "derive_repo_constitution", {
        files: [
          { path: ".github/pull_request_template.md", lines: 4 },
          { path: ".github/workflows/ci.yml", lines: 3 },
          { path: "package.json", lines: 1 },
          { path: "index.ts", lines: 1 }
        ],
        artifacts: [
          { path: ".github/pull_request_template.md", content: "## Summary\n\n## Verification\n" },
          { path: ".github/workflows/ci.yml", content: "name: CI\non:\n  pull_request:\n" },
          { path: "package.json", content: JSON.stringify({ name: "demo", scripts: { test: "node --test" } }) }
        ],
        recentPullRequests: [
          { number: 5, author: "maintainer", merged: true, body: "## What?\n\n## Test Plan\n" }
        ]
      });
      assert.equal(supplied.constitution.schemaVersion, 1);
      assert.equal(supplied.constitution.pullRequests.templates[0].path, ".github/pull_request_template.md");
      assert.equal(supplied.constitution.pullRequests.recentStyle.advisory, true);
      assert.equal(supplied.constitution.publicSafety.rawContentIncluded, false);

      const local = await callJson(client, "derive_local_repo_constitution", {
        rootPath: root,
        allowOutsideCwd: true,
        recentPullRequests: [
          { number: 6, author: "dependabot[bot]", merged: true, body: "## Dependencies\n" }
        ]
      });
      assert.equal(local.filesReviewed >= 5, true);
      assert.equal(local.constitution.instructions.agentInstructionPaths.includes("AGENTS.md"), true);
      assert.equal(local.constitution.packageMetadata[0].name, "demo");
      assert.equal(local.constitution.pullRequests.recentStyle.botSamplesIgnored, 1);
      assert.equal(JSON.stringify(local.constitution).includes("Use the local work gate"), false);
    } finally {
      rmSync(root, { recursive: true, force: true });
      await close();
    }
  });
});

async function connectTestClient() {
  const server = createArchitectServer({ enableLocalWorkspaceTool: true, toolSurface: "advanced" });
  const client = new Client({ name: "architect-mcp-foundry-test-client", version: "0.1.0" });
  const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
  await Promise.all([
    client.connect(clientTransport),
    server.connect(serverTransport)
  ]);
  return {
    client,
    close: async () => {
      await client.close();
      await server.close();
    }
  };
}

async function callJson(client: Client, name: string, args: Record<string, unknown>) {
  const result = await client.callTool({ name, arguments: args });
  const text = result.content.find((content) => content.type === "text")?.text;
  assert.equal(typeof text, "string");
  return JSON.parse(text as string);
}
