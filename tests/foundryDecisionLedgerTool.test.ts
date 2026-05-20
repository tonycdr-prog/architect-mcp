import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";

describe("Foundry decision ledger MCP tool", () => {
  it("routes scored MCP output into a non-mutating public-safe ledger", async () => {
    const { client, close } = await connectTestClient();
    try {
      const normalized = await callJson(client, "normalize_foundry_evidence", {
        findings: [{
          code: "ARCH008_ENV_SCATTER",
          confidence: "high",
          severity: "error",
          path: "src/config/env.ts",
          message: "Environment access is scattered.",
          recommendation: "Centralize environment parsing."
        }],
        verification: [{ check: "npm test", status: "passed" }]
      });
      const scored = await callJson(client, "score_foundry_actionability", {
        inventory: normalized.inventory
      });
      const routed = await callJson(client, "route_foundry_decisions", {
        actionability: scored.actionability,
        ledgerId: "slice-322",
        recordedAt: "2026-05-20T00:00:00.000Z"
      });

      assert.equal(routed.ledger.schemaVersion, 1);
      assert.equal(routed.ledger.ledgerId, "foundry-ledger");
      assert.equal(routed.ledger.entries[0].route, "pr_preview");
      assert.deepEqual(routed.ledger.entries[0].evidenceIds, ["fde-001"]);
      assert.deepEqual(routed.ledger.entries[0].source, { score: 81 });
      assert.equal(Object.hasOwn(routed.ledger.entries[0].source, "actionabilityDecision"), false);
      assert.equal(routed.ledger.entries[0].mutation.serverMutationAllowed, false);
      assert.equal(routed.ledger.publicSafety.rawPayloadsIncluded, false);
      assert.equal(routed.ledger.summary.serverWritesPerformed, 0);
    } finally {
      await close();
    }
  });

  it("replays the slice smoke with every route and public-safety holds", async () => {
    const { client, close } = await connectTestClient();
    try {
      const reports = await Promise.all([
        scoreInventory(client, [{
          code: "ARCH008_ENV_SCATTER",
          confidence: "high",
          severity: "error",
          path: "src/config/env.ts",
          message: "Environment access is scattered.",
          recommendation: "Centralize environment parsing."
        }], [{ check: "npm test", status: "passed" }]),
        scoreInventory(client, [{
          code: "ARCH008_ENV_SCATTER",
          confidence: "high",
          severity: "error",
          path: "src/runtime/options.ts",
          message: "Runtime option parsing is duplicated.",
          recommendation: "Route option parsing through a single policy module."
        }], []),
        scoreInventory(client, [{
          code: "ARCH001_OVERSIZED_FILE",
          confidence: "medium",
          severity: "warning",
          path: "docs/examples/generated-client.ts",
          message: "Docs example generated client is large.",
          recommendation: "Treat this as a docs-example suppression candidate."
        }], [{ check: "npm test", status: "passed" }]),
        scoreInventory(client, [], [{ check: "npm test", status: "passed" }], [{
          toolName: "style-scanner",
          ruleId: "STYLE_NIT",
          confidence: "high",
          severity: "info",
          path: "scripts/style-nit.ts",
          message: "Minor style suggestion.",
          recommendation: "No code change required."
        }]),
        scoreInventory(client, [], [{ check: "npm test", status: "passed" }], [{
          toolName: "security-scanner",
          ruleId: "SECURITY_REVIEW_HOLD",
          confidence: "high",
          severity: "error",
          path: "src/security/redacted.ts",
          message: "Security-sensitive finding requires maintainer disclosure review.",
          recommendation: "Ask a human maintainer before public routing.",
          rawPayload: { payload: "private details omitted" },
          securitySensitive: true
        }])
      ]);
      const assessments = reports.flatMap((report) => report.actionability.assessments);
      const routed = await callJson(client, "route_foundry_decisions", {
        ledgerId: "/Users/alice/project/npm_abcdefghijklmnopqrstuvwxyz1234567890",
        recordedAt: "stdout: private timestamp",
        actionability: {
          schemaVersion: 1,
          summary: {
            totalFindings: assessments.length,
            byDecision: {},
            prPreviewCandidates: 0,
            askHuman: 0,
            exceptionCandidates: 0,
            noOpCandidates: 0,
            publicSafetyHolds: 0
          },
          assessments,
          publicSafety: {
            rawPayloadsIncluded: false,
            rawRepoContentIncluded: false,
            localPathsIncluded: false,
            tokenValuesIncluded: false,
            mutationAllowed: false,
            publicRecommendationsOnly: true
          }
        }
      });
      const serialized = JSON.stringify(routed.ledger);

      assert.deepEqual(routed.ledger.summary.byRoute, {
        pr_preview: 1,
        architect_issue: 1,
        exception: 1,
        no_op: 1,
        ask_human: 1
      });
      assert.equal(routed.ledger.publicSafety.rawPayloadsIncluded, false);
      assert.equal(routed.ledger.publicSafety.rawRepoContentIncluded, false);
      assert.equal(routed.ledger.publicSafety.localPathsIncluded, false);
      assert.equal(routed.ledger.publicSafety.tokenValuesIncluded, false);
      assert.equal(routed.ledger.publicSafety.mutationAllowed, false);
      assert.equal(routed.ledger.summary.serverWritesPerformed, 0);
      assert.equal(routed.ledger.ledgerId, "foundry-ledger");
      assert.equal(serialized.includes("/Users/alice"), false);
      assert.equal(serialized.includes("npm_abcdefghijklmnopqrstuvwxyz1234567890"), false);
      assert.equal(serialized.includes("private"), false);
      assert.equal(serialized.includes("fev-"), false);
      assert.equal(serialized.includes("actionabilityDecision"), false);
      assert.equal(serialized.includes("\"path\""), false);
    } finally {
      await close();
    }
  });
});

async function connectTestClient() {
  const server = createArchitectServer({ enableLocalWorkspaceTool: true, toolSurface: "advanced" });
  const client = new Client({ name: "architect-mcp-foundry-ledger-test-client", version: "0.1.0" });
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

async function scoreInventory(
  client: Client,
  findings: Array<Record<string, unknown>>,
  verification: Array<Record<string, unknown>>,
  externalFindings: Array<Record<string, unknown>> = []
) {
  const normalized = await callJson(client, "normalize_foundry_evidence", {
    findings,
    externalFindings,
    verification
  });
  return callJson(client, "score_foundry_actionability", {
    inventory: normalized.inventory
  });
}
