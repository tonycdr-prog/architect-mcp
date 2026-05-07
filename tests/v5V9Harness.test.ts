import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";
import { createReviewReport } from "../src/domain/reviewReport.js";
import { generateContract } from "../src/domain/contract.js";
import { runV5V9EvalHarness } from "../src/domain/v5V9EvalHarness.js";
import { cleanMcpServerFixture, messyReactFixture } from "./fixtures/repos.js";

describe("V5-V9 MCP-driven implementation surface", () => {
  it("runs staged V5-V9 eval harnesses", () => {
    for (const stage of ["v5", "v6", "v7", "v8", "v9"] as const) {
      const report = runV5V9EvalHarness(stage);
      assert.equal(report.status, "pass", `${stage} should pass`);
      assert.equal(report.summary.failed, 0);
    }
  });

  it("exposes V5-V9 tools through MCP with stable local-first shapes", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      for (const name of [
        "resolve_standards_profile",
        "explain_review_findings",
        "simulate_policy_gate",
        "analyze_standards_conflicts",
        "score_repo_profile_fit",
        "review_contract_lifecycle",
        "list_policy_bundles",
        "validate_policy_bundles",
        "preview_policy_bundle",
        "summarize_session_continuity",
        "cluster_review_findings",
        "generate_local_report_artifact",
        "analyze_regression_coverage",
        "review_pattern_card",
        "preview_pattern_card",
        "create_architecture_strategy_map",
        "compare_standards_profiles",
        "preview_change_what_if",
        "diagnose_agent_behavior",
        "draft_rule_candidate",
        "compare_review_trends",
        "render_governance_pack",
        "review_standards_refactor",
        "minimize_policy_set",
        "select_review_playbook",
        "review_playbook_conformance",
        "check_agent_collaboration_plan",
        "run_failure_mode_drills",
        "calibrate_rule_impact",
        "review_documentation_intelligence",
        "select_local_orchestration_recipe",
        "evaluate_scenario_acceptance",
        "normalize_mcp_result",
        "plan_context_budget",
        "route_evidence",
        "create_local_dry_run_plan",
        "review_tool_loop_quality",
        "run_v9_eval_harness"
      ]) {
        assert.equal(tools.tools.some((tool) => tool.name === name), true, `${name} missing`);
      }

      const contract = generateContract(cleanMcpServerFixture.brief, ["mcp-server"]);
      const finding = {
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "high",
        severity: "warning",
        path: "src/domain/large.ts",
        message: "File has 900 lines.",
        recommendation: "Split the file by responsibility."
      };
      const report = createReviewReport([finding]);

      assert.equal((await callJson(client, "resolve_standards_profile", { request: { brief: cleanMcpServerFixture.brief } })).profile, "balanced");
      assert.equal((await callJson(client, "explain_review_findings", { request: { findings: [finding] } })).explanations[0].fixShape, "split file");
      assert.equal((await callJson(client, "simulate_policy_gate", { request: { findings: [finding] } })).simulations.length, 4);
      assert.equal(Array.isArray((await callJson(client, "analyze_standards_conflicts", {})).conflicts), true);
      assert.equal((await callJson(client, "score_repo_profile_fit", { request: { brief: cleanMcpServerFixture.brief, files: cleanMcpServerFixture.files } })).profiles.length > 0, true);
      assert.equal((await callJson(client, "review_contract_lifecycle", { request: { after: contract, findings: [finding] } })).maturity.length > 0, true);

      assert.equal((await callJson(client, "validate_policy_bundles", {})).valid, true);
      assert.equal((await callJson(client, "preview_policy_bundle", { request: { brief: cleanMcpServerFixture.brief, findings: [finding] } })).coverage.rules > 0, true);
      assert.equal((await callJson(client, "summarize_session_continuity", { request: { summaries: ["changed V5"], assumptions: [{ statement: "local only" }] } })).intentDebt, 1);
      assert.equal((await callJson(client, "cluster_review_findings", { request: { findings: [finding] } })).groups.length, 1);
      assert.match((await callJson(client, "generate_local_report_artifact", { request: { report } })).markdown, /Priority Findings/);
      assert.equal(Array.isArray((await callJson(client, "analyze_regression_coverage", {})).gaps), true);
      assert.equal((await callJson(client, "review_pattern_card", { request: { card: { id: "local-pattern", summary: "Keep local agent harness outputs compact and explicit." } } })).valid, true);
      assert.equal((await callJson(client, "preview_pattern_card", { request: { card: { id: "local-pattern", summary: "Keep local agent harness outputs compact and explicit." } } })).applies, true);

      assert.equal(Array.isArray((await callJson(client, "create_architecture_strategy_map", { request: { brief: cleanMcpServerFixture.brief } })).standards), true);
      assert.equal((await callJson(client, "compare_standards_profiles", {})).profiles.some((profile: { profile: string }) => profile.profile === "strict"), true);
      assert.equal((await callJson(client, "preview_change_what_if", { request: { files: [{ path: "src/auth/session.ts", purpose: "auth refactor" }] } })).blastRadius, "high");
      assert.equal((await callJson(client, "diagnose_agent_behavior", { request: { sessionSummaries: ["skipped verification"] } })).patterns.includes("skipped verification"), true);
      assert.equal((await callJson(client, "draft_rule_candidate", { request: { sourceText: "UI files must not import server database modules." } })).classification, "executable");
      assert.equal(Array.isArray((await callJson(client, "compare_review_trends", { request: { before: report, after: createReviewReport([]) } })).resolvedFindings), true);
      assert.match((await callJson(client, "render_governance_pack", {})).markdown, /Governance Pack/);

      assert.equal(Array.isArray((await callJson(client, "review_standards_refactor", { request: { stackPacks: contract.stackPacks } })).suggestions), true);
      assert.equal((await callJson(client, "minimize_policy_set", { request: { brief: cleanMcpServerFixture.brief } })).compactProfile.length > 0, true);
      assert.equal((await callJson(client, "select_review_playbook", { request: { request: "fix auth bug" } })).playbook.id, "security-sensitive-change");
      assert.equal((await callJson(client, "review_playbook_conformance", { request: { toolsRun: ["interpret_implementation_intent"], verification: [] } })).valid, false);
      assert.equal((await callJson(client, "check_agent_collaboration_plan", { request: { ownership: [{ agent: "a", files: ["src/a.ts"] }, { agent: "b", files: ["src/a.ts"] }] } })).valid, false);
      assert.equal((await callJson(client, "check_agent_collaboration_plan", { request: { ownership: [{ agent: "a", files: ["src/domain/private/Secret.ts"] }], doNotTouch: ["src/domain/private/**"] } })).valid, false);
      assert.equal((await callJson(client, "run_failure_mode_drills", {})).status, "pass");
      assert.match((await callJson(client, "calibrate_rule_impact", { request: { findings: [finding] } })).previewGateChange, /warn|fail|pass|strict/);
      assert.equal((await callJson(client, "review_documentation_intelligence", { request: { readme: "hello", toolNames: ["missing_tool"] } })).status, "warn");

      assert.equal((await callJson(client, "select_local_orchestration_recipe", { request: { request: "review existing repo" } })).recipe.tools.length > 0, true);
      assert.equal((await callJson(client, "select_local_orchestration_recipe", { request: { request: "fix auth bug" } })).recipe.id, "security-sensitive-change");
      assert.equal((await callJson(client, "evaluate_scenario_acceptance", { request: { verified: true } })).status, "pass");
      assert.equal((await callJson(client, "normalize_mcp_result", { request: { findings: [finding], evidence: ["npm test passed"] } })).stoplight, "yellow");
      assert.equal((await callJson(client, "plan_context_budget", { request: { mode: "compact", findings: [finding] } })).mode, "compact");
      assert.equal((await callJson(client, "route_evidence", { request: { findings: [finding], sources: [{ id: "hono", snapshotPath: "stack-sources/ingested/hono.md", sha256: "abc" }] } })).evidence[0].id, "ev-1");
      assert.equal((await callJson(client, "create_local_dry_run_plan", { request: { request: "risky auth refactor", risky: true } })).gates.includes("pre-edit contract"), true);
      assert.equal((await callJson(client, "review_tool_loop_quality", { request: { risky: true, toolsRun: [], verification: [] } })).status, "fail");
      assert.match((await callJson(client, "review_tool_loop_quality", { request: { risky: false, toolsRun: ["review_repo_structure"], verification: [{ status: "passed" }], finalResponse: "Changed files only." } })).findings[0], /verified|assumptions|not done/);
      assert.equal((await callJson(client, "run_v9_eval_harness", {})).status, "pass");
    } finally {
      await close();
    }
  });

  it("keeps hosted-mode local workspace scanning disabled while allowing V10 productization tools", async () => {
    const { client, close } = await connectTestClient(false);
    try {
      const tools = await client.listTools();
      assert.equal(tools.tools.some((tool) => tool.name === "review_local_workspace"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "get_v10_productization_blueprint"), true);
      assert.equal((await callJson(client, "preview_change_what_if", { request: { files: messyReactFixture.files.map((file) => ({ path: file.path, purpose: "existing fixture file" })) } })).blastRadius.length > 0, true);
    } finally {
      await close();
    }
  });
});

async function connectTestClient(enableLocalWorkspaceTool = true) {
  const server = createArchitectServer({ enableLocalWorkspaceTool });
  const client = new Client({ name: "architect-mcp-v5-v9-test-client", version: "0.1.0" });
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
