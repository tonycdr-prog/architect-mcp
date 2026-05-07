import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { auditGeneratedRepoQuality, buildQualityRequirementsProfile, evaluateRepoPlanQuality, runRepoQualityEvalScenarios, suggestQualityFollowUpQuestions } from "../src/domain/repoQualityEval.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";

describe("repo quality eval layer", () => {
  it("builds a requirements profile and asks useful missing questions", () => {
    const profile = buildQualityRequirementsProfile({
      userLevel: "beginner",
      answers: ["I want a simple app for admins."]
    });

    assert.equal(profile.userLevel, "beginner");
    assert.equal(profile.confidence, "low");
    assert.equal(profile.missingQuestions.some((question) => /data/i.test(question)), true);
  });

  it("blocks secrets, unsafe permissions, and destructive commands", () => {
    const result = evaluateRepoPlanQuality({
      plan: {
        permissions: ["admin:*"],
        destructiveCommands: ["rm -rf ./data"]
      },
      signals: {
        hasHardcodedSecrets: true
      }
    });

    assert.equal(result.decision, "block");
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG001_COMMITTED_SECRET"), true);
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG005_DESTRUCTIVE_COMMAND"), true);
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG006_UNSAFE_PERMISSIONS"), true);
  });

  it("does not reward fake CI or trivial tests", () => {
    const result = evaluateRepoPlanQuality({
      plan: {
        ciCommands: ["echo passed"],
        testDescriptions: ["expect(true).toBe(true)"]
      },
      signals: {
        hasMeaningfulCi: true,
        ciOnlyEchoes: true,
        hasMeaningfulTests: false,
        testsAreTrivial: true
      }
    });

    assert.equal(result.decision, "fix_before_generate");
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG003_FAKE_CI"), true);
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG004_FAKE_TESTS"), true);
    assert.equal(result.antiRewardHackingWarnings.some((warning) => /CI passing is not enough/.test(warning)), true);
  });

  it("requires env examples, setup docs, and AGENTS.md when relevant", () => {
    const result = evaluateRepoPlanQuality({
      plan: {
        envVars: ["DATABASE_URL"],
        docs: ["README.md"]
      },
      signals: {
        hasReadme: true,
        hasSetupInstructions: false,
        hasEnvExample: false,
        hasAgentsMd: false
      }
    });

    assert.equal(result.decision, "fix_before_generate");
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG002_ENV_EXAMPLE_MISSING"), true);
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG007_SETUP_DOCS_MISSING"), true);
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG008_AGENTS_MD_WEAK"), true);
  });

  it("merges inferred plan signals with partial caller signals", () => {
    const result = evaluateRepoPlanQuality({
      plan: {
        envVars: ["OPENAI_API_KEY=sk-test-secret-value"],
        ciCommands: ["echo passed"]
      },
      signals: {
        hasReadme: true
      }
    });

    assert.equal(result.hardGates.some((gate) => gate.code === "RQG001_COMMITTED_SECRET"), true);
    assert.equal(result.hardGates.some((gate) => gate.code === "RQG003_FAKE_CI"), true);
  });

  it("detects common secret-shaped env values and missing generated-repo docs", () => {
    const secretResult = evaluateRepoPlanQuality({
      plan: {
        envVars: ["DATABASE_URL=postgres://user:pass@localhost/db", "SESSION_SECRET=super-secret-value"]
      }
    });
    const auditResult = auditGeneratedRepoQuality({
      profile: {
        userLevel: "beginner",
        goals: ["Build a customer intake app for admin users"],
        constraints: ["Runs on web", "Stores customer data", "Success means users can create intakes"],
        knownRisks: [],
        missingQuestions: [],
        confidence: "high"
      },
      plan: {
        stack: ["React", "Express", "Postgres"],
        architecture: "Feature modules with server-owned repositories and UI components separated from data access.",
        files: ["src/features/intake/index.ts", "src/server/repositories/intake.ts", "tests/intake.test.ts"],
        ciCommands: ["npm test"],
        testDescriptions: ["creates an intake record"]
      }
    });

    assert.equal(secretResult.hardGates.some((gate) => gate.code === "RQG001_COMMITTED_SECRET"), true);
    assert.equal(auditResult.hardGates.some((gate) => gate.code === "RQG007_SETUP_DOCS_MISSING"), true);
    assert.equal(auditResult.hardGates.some((gate) => gate.code === "RQG008_AGENTS_MD_WEAK"), true);
  });

  it("keeps env values intact when secret values contain additional equals signs", () => {
    const result = evaluateRepoPlanQuality({
      plan: {
        envVars: ["SESSION_SECRET=abc=def=ghi"]
      }
    });

    assert.equal(result.hardGates.some((gate) => gate.code === "RQG001_COMMITTED_SECRET"), true);
  });

  it("passes a boring maintainable plan with real proof", () => {
    const profile = buildQualityRequirementsProfile({
      userLevel: "beginner",
      goals: ["Build a web app for admin users to manage customer intake data"],
      constraints: ["Stores data", "Runs on the web", "Success means admins can create and review intakes"]
    });
    const result = evaluateRepoPlanQuality({
      profile,
      plan: {
        stack: ["React", "Express", "Postgres"],
        architecture: "Feature modules with server-owned repositories and UI components separated from data access.",
        files: ["src/features/intake", "src/server/repositories", "tests/intake.test.ts"],
        ciCommands: ["npm run typecheck", "npm test", "npm run build"],
        testDescriptions: ["Creates an intake and validates required fields"],
        docs: ["README.md setup/run/test", ".env.example", "AGENTS.md"],
        envVars: ["DATABASE_URL"],
        tradeoffs: ["Boring stack is easier to maintain than a distributed architecture."],
        explanations: ["This uses common tools so another developer can pick it up later."]
      },
      signals: {
        hasReadme: true,
        hasSetupInstructions: true,
        hasEnvExample: true,
        hasAgentsMd: true,
        hasMeaningfulCi: true,
        hasMeaningfulTests: true,
        testsAreTrivial: false,
        explainsTradeoffs: true
      }
    });

    assert.equal(result.decision, "proceed");
    assert.equal(result.stoplight, "green");
    assert.equal(result.overallScore >= 80, true);
  });

  it("suggests focused follow-up questions instead of a questionnaire", () => {
    const result = suggestQualityFollowUpQuestions({
      profile: buildQualityRequirementsProfile({ goals: [] })
    });

    assert.equal(result.questions.length <= 3, true);
    assert.equal(result.confidence, "low");
  });

  it("runs deterministic quality scenarios", () => {
    const report = runRepoQualityEvalScenarios();

    assert.equal(report.status, "pass");
    assert.equal(report.summary.failed, 0);
  });

  it("exposes repo quality tools through MCP", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      for (const name of ["build_quality_requirements_profile", "evaluate_repo_plan_quality", "audit_generated_repo_quality", "suggest_quality_followup_questions", "run_repo_quality_eval_scenarios"]) {
        assert.equal(tools.tools.some((tool) => tool.name === name), true, `${name} missing`);
      }

      const profile = await callJson(client, "build_quality_requirements_profile", { request: { answers: ["A small admin app"] } });
      assert.equal(profile.confidence, "low");

      const evaluation = await callJson(client, "evaluate_repo_plan_quality", {
        request: {
          signals: {
            hasHardcodedSecrets: true
          }
        }
      });
      assert.equal(evaluation.decision, "block");

      assert.equal((await callJson(client, "run_repo_quality_eval_scenarios", {})).status, "pass");
    } finally {
      await close();
    }
  });
});

async function connectTestClient() {
  const server = createArchitectServer();
  const client = new Client({ name: "architect-mcp-quality-test-client", version: "0.1.0" });
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
  const response = await client.callTool({ name, arguments: args });
  const text = response.content?.[0]?.type === "text" ? response.content[0].text : "{}";
  return JSON.parse(text);
}
