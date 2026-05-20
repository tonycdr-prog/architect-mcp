import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";

describe("Foundry MCP tools", () => {
  it("runs the offline Foundry eval corpus through the MCP surface", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      assert.equal(tools.tools.some((tool) => tool.name === "run_foundry_eval_corpus"), true);

      const report = await callJson(client, "run_foundry_eval_corpus", {});

      assert.equal(report.status, "pass");
      assert.equal(report.summary.offlineNetworkRequired, false);
      assert.equal(report.summary.serverWritesPerformed, 0);
      assert.equal(report.requirements.routeCoverage.pr_preview, true);
      assert.equal(report.requirements.routeCoverage.architect_issue, true);
      assert.equal(report.requirements.routeCoverage.exception, true);
      assert.equal(report.requirements.routeCoverage.no_op, true);
      assert.equal(report.requirements.publicSafety, true);
      assert.equal(JSON.stringify(report).includes("/Users/"), false);
      assert.equal(JSON.stringify(report).includes("RAW_PRIVATE_PAYLOAD"), false);
    } finally {
      await close();
    }
  });

  it("normalizes supplied evidence through the MCP surface", async () => {
    const { client, close } = await connectTestClient();
    try {
      const normalized = await callJson(client, "normalize_foundry_evidence", {
        findings: [
          {
            code: "ARCH001_OVERSIZED_FILE",
            confidence: "medium",
            severity: "warning",
            path: "vendor/generated-client.ts",
            message: "File has 1300 lines and may be too large.",
            recommendation: "Check whether this generated vendor file should be suppressed."
          }
        ],
        externalFindings: [
          {
            toolName: "scanner",
            message: "Raw payload included stdout=/tmp/private/log.txt",
            rawPayload: { stdout: "private" }
          }
        ],
        verification: [
          { check: "npm test", status: "passed", summary: "passed" }
        ]
      });

      assert.equal(normalized.inventory.schemaVersion, 1);
      assert.equal(normalized.inventory.publicSafety.rawPayloadsIncluded, false);
      assert.equal(normalized.inventory.publicSafety.rawRepoContentIncluded, false);
      assert.equal(normalized.inventory.publicSafety.localPathsIncluded, false);
      assert.equal(normalized.inventory.publicSafety.tokenValuesIncluded, false);
      assert.equal(normalized.inventory.publicSafety.mutationAllowed, false);
      assert.equal(normalized.inventory.summary.omittedRawPayloads, 1);
      assert.equal(normalized.inventory.evidence.some((item: { suppressionCandidate?: { category: string } }) => item.suppressionCandidate?.category === "vendored_code"), true);
      assert.equal(JSON.stringify(normalized.inventory).includes("private"), false);
    } finally {
      await close();
    }
  });

  it("scores normalized evidence through the MCP surface", async () => {
    const { client, close } = await connectTestClient();
    try {
      const normalized = await callJson(client, "normalize_foundry_evidence", {
        findings: [
          {
            code: "ARCH008_ENV_SCATTER",
            confidence: "high",
            severity: "error",
            path: "src/config/env.ts",
            message: "Environment access is scattered.",
            recommendation: "Centralize environment parsing."
          }
        ],
        verification: [
          { check: "npm test", status: "passed", summary: "passed" }
        ]
      });
      const scored = await callJson(client, "score_foundry_actionability", {
        inventory: normalized.inventory
      });

      assert.equal(scored.actionability.schemaVersion, 1);
      assert.equal(scored.actionability.publicSafety.mutationAllowed, false);
      assert.equal(scored.actionability.assessments[0].decision, "pr_preview_candidate");
      assert.equal(scored.actionability.assessments[0].requiredVerification.some((item: string) => item.includes("npm test")), true);
    } finally {
      await close();
    }
  });

  it("redacts caller-supplied unsafe inventory fields before scoring through MCP", async () => {
    const { client, close } = await connectTestClient();
    try {
      const scored = await callJson(client, "score_foundry_actionability", {
        inventory: {
          schemaVersion: 1,
          summary: { totalEvidence: 1, bySourceType: {}, byConfidence: {}, byPublicSafetyClass: {}, redacted: 0, omittedRawPayloads: 0, suppressionCandidates: 0, coverageCaveats: 0 },
          evidence: [{
            id: "/Users/alice/project/npm_abcdefghijklmnopqrstuvwxyz1234567890",
            kind: "finding",
            sourceType: "external_tool",
            sourceRef: {
              sourceType: "external_tool",
              sourceId: "npm_abcdefghijklmnopqrstuvwxyz1234567890",
              path: "/Users/alice/project/.env"
            },
            confidence: "high",
            severity: "error",
            code: "npm_abcdefghijklmnopqrstuvwxyz1234567890",
            path: "/Users/alice/project/.env",
            publicSummary: "Caller supplied identity fields were not normalized.",
            publicSafetyClass: "public",
            redactionStatus: "none"
          }],
          coverage: { scanTruncated: false, detailedFindingsTruncated: false, topScannedDirectories: [], findingHistogram: [], caveats: [] },
          suppressionPrerequisites: [],
          publicSafety: {
            rawPayloadsIncluded: false,
            rawRepoContentIncluded: false,
            localPathsIncluded: false,
            tokenValuesIncluded: false,
            mutationAllowed: false
          }
        }
      });
      const serialized = JSON.stringify(scored);

      assert.equal(scored.actionability.assessments[0].decision, "ask_human");
      assert.equal(scored.actionability.assessments[0].sourceType, "external_tool");
      assert.equal(scored.actionability.assessments[0].blockers.some((item: string) => item.includes("identity fields needed redaction")), true);
      assert.equal(serialized.includes("/Users/alice"), false);
      assert.equal(serialized.includes("npm_abcdefghijklmnopqrstuvwxyz1234567890"), false);
    } finally {
      await close();
    }
  });

  it("keeps raw-output-shaped verification labels out of PR-preview routing", async () => {
    const { client, close } = await connectTestClient();
    try {
      const normalized = await callJson(client, "normalize_foundry_evidence", {
        findings: [
          {
            code: "ARCH008_ENV_SCATTER",
            confidence: "high",
            severity: "error",
            path: "src/config/env.ts",
            message: "Environment access is scattered.",
            recommendation: "Centralize environment parsing."
          }
        ],
        verification: [
          { check: "npm test stderr: private stack trace line", status: "passed", summary: "passed" }
        ]
      });
      const scored = await callJson(client, "score_foundry_actionability", {
        inventory: normalized.inventory
      });
      const serialized = JSON.stringify(scored);

      assert.equal(scored.actionability.assessments[0].decision, "ask_human");
      assert.equal(scored.actionability.assessments[0].blockers.some((item: string) => item.includes("passing verification path is missing")), true);
      assert.equal(serialized.includes("private stack trace"), false);
      assert.equal(scored.actionability.assessments[0].requiredVerification.some((item: string) => item.includes("[redacted-raw-output]")), true);
    } finally {
      await close();
    }
  });

  it("rejects malformed repo constitution inputs before normalization", async () => {
    const { client, close } = await connectTestClient();
    try {
      const result = await callToolRaw(client, "normalize_foundry_evidence", {
        repoConstitution: {}
      });

      assert.equal(result.isError, true);
    } finally {
      await close();
    }
  });

  it("accepts partially shaped review-report coverage payloads while rejecting unknown external/verification fields", async () => {
    const { client, close } = await connectTestClient();
    try {
      const acceptsCoverage = await callJson(client, "normalize_foundry_evidence", {
        reviewReports: [
          {
            coverage: {
              filesReviewed: 2,
              maxFiles: 5,
              topScannedDirectories: [{ directory: "src", files: 2 }],
              findingHistogram: [{ code: "ARCH001_OVERSIZED_FILE", severity: "warning", count: 1 }],
              caveats: ["coverage payload supplied by caller"],
              callerSpecificMetadata: { source: "integration" }
            }
          }
        ]
      });
      assert.equal(acceptsCoverage.inventory.coverage.filesReviewed, 2);
      assert.equal(acceptsCoverage.inventory.coverage.caveats.includes("coverage payload supplied by caller"), true);

      const rejectExternalUnknownField = await callToolRaw(client, "normalize_foundry_evidence", {
        externalFindings: [
          {
            toolName: "scanner",
            message: "unexpected key",
            unexpected: "nope"
          }
        ]
      });
      assert.equal(rejectExternalUnknownField.isError, true);

      const rejectVerificationUnknownField = await callToolRaw(client, "normalize_foundry_evidence", {
        verification: [
          {
            check: "npm test",
            status: "passed",
            extra: "nope"
          }
        ]
      });
      assert.equal(rejectVerificationUnknownField.isError, true);
    } finally {
      await close();
    }
  });

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
  const result = await callToolRaw(client, name, args);
  const text = result.content.find((content) => content.type === "text")?.text;
  assert.equal(typeof text, "string");
  return JSON.parse(text as string);
}

async function callToolRaw(client: Client, name: string, args: Record<string, unknown>) {
  return client.callTool({ name, arguments: args });
}
