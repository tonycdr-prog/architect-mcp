import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createArchitectServer } from "../src/server/createArchitectServer.js";
import { createBaselineFromFindings } from "../src/domain/baseline.js";
import { generateContract } from "../src/domain/contract.js";
import { reviewFileSummaries } from "../src/domain/reviewer.js";
import { generateToolReferenceMarkdown } from "../src/domain/toolReferenceDocs.js";
import { messyReactFixture, cleanMcpServerFixture, publicRepoSmokeFixture } from "./fixtures/repos.js";
import { CORE_ARCHITECTURE_TOOL_NAMES } from "../src/tools/toolRegistry.js";

describe("MCP tool responses", () => {
  it("exposes only the core agent work-gate tools by default", async () => {
    const server = createArchitectServer();
    const client = new Client({ name: "architect-mcp-core-test-client", version: "0.1.0" });
    const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
    await Promise.all([
      client.connect(clientTransport),
      server.connect(serverTransport)
    ]);
    try {
      const tools = await client.listTools();
      assert.deepEqual(tools.tools.map((tool) => tool.name).sort(), [...CORE_ARCHITECTURE_TOOL_NAMES].sort());
      assert.equal(tools.tools.some((tool) => tool.name === "promote_stack_pack_to_files"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "review_local_workspace"), false);
    } finally {
      await client.close();
      await server.close();
    }
  });

  it("runs the default core agent work-gate golden path", async () => {
    const server = createArchitectServer();
    const client = new Client({ name: "architect-mcp-core-golden-path-client", version: "0.1.0" });
    const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
    await Promise.all([
      client.connect(clientTransport),
      server.connect(serverTransport)
    ]);
    try {
      const preEdit = await callJson(client, "create_pre_edit_contract", {
        input: {
          request: "make auth better with best practices",
          stack: {
            backend: "TypeScript MCP server"
          },
          verification: ["npm test"]
        },
        likelyFiles: ["src/server/auth.ts", "tests/auth.test.ts"],
        verificationChecks: ["npm test"]
      });
      assert.equal(preEdit.contract.verificationChecks[0], "npm test");
      assert.equal(preEdit.intent.decision, "confirm_before_edit");

      const buildPlanReview = await callJson(client, "review_build_plan", {
        allowedChecks: ["npm test"],
        plan: {
          archetype: "ai-workflow-tool",
          slices: [
            {
              id: "agent-harness",
              title: "Confirm agent harness",
              order: 1,
              goal: "Confirm the pre-edit contract and verification evidence before implementation.",
              inputs: ["Issue #102 pre-edit contract"],
              outputs: ["Scoped implementation handoff"],
              allowedDirectories: ["docs"],
              forbiddenFiles: ["src/App.tsx", "src/index.ts", "src/server.ts"],
              files: ["docs/build-plan.md"],
              checks: ["architecture contract validates"],
              stopAfter: "Stop if the contract is not accepted."
            },
            {
              id: "backend-boundary",
              title: "Implement auth boundary",
              order: 2,
              goal: "Keep auth behavior inside server-owned modules.",
              inputs: ["Pre-edit contract"],
              outputs: ["Auth boundary implementation and tests"],
              allowedDirectories: ["src/server", "tests"],
              forbiddenFiles: ["src/App.tsx", "src/index.ts", "src/server.ts"],
              files: ["src/server/auth.ts", "tests/auth.test.ts"],
              checks: ["npm test"],
              stopAfter: "Stop after npm test passes."
            }
          ]
        }
      });
      assert.equal(buildPlanReview.summary.errors, 0);

      const proposedPlanReview = await callJson(client, "review_proposed_file_plan", {
        plan: {
          files: [
            {
              path: "AGENTS.md",
              purpose: "Capture local agent work-gate rules.",
              responsibilities: ["agent instructions"]
            },
            {
              path: "docs/architecture-contract.md",
              purpose: "Document the pre-edit architecture contract.",
              responsibilities: ["contract documentation"]
            },
            {
              path: "src/server/auth.ts",
              purpose: "Own server-side auth validation.",
              responsibilities: ["session validation"]
            },
            {
              path: "tests/auth.test.ts",
              purpose: "Verify auth boundary behavior.",
              responsibilities: ["auth regression tests"]
            }
          ]
        }
      });
      assert.equal(proposedPlanReview.summary.errors, 0);

      const implementationReview = await callJson(client, "review_implementation_against_contract", {
        input: {
          contract: preEdit.contract,
          changedFiles: [
            { path: "src/server/auth.ts", lines: 80 },
            { path: "tests/auth.test.ts", lines: 60 }
          ],
          verification: [
            { check: "npm test", status: "passed" }
          ]
        }
      });
      assert.equal(implementationReview.valid, true);

      const finalResponse = "Changed src/server/auth.ts and tests/auth.test.ts. Verified npm test passed. Assumptions: auth remains server-owned. Not done: no schema or public API changes.";
      const finalReview = await callJson(client, "review_agent_final_response", {
        request: {
          response: finalResponse,
          requiredChecks: ["npm test"]
        }
      });
      assert.equal(finalReview.valid, true);

      const sessionReview = await callJson(client, "review_agent_session", {
        request: {
          intent: preEdit.intent,
          contract: preEdit.contract,
          changedFiles: [
            { path: "src/server/auth.ts", lines: 80 },
            { path: "tests/auth.test.ts", lines: 60 }
          ],
          verification: [
            { check: "npm test", status: "passed" }
          ],
          finalResponse
        }
      });
      assert.notEqual(sessionReview.status, "fail");
    } finally {
      await client.close();
      await server.close();
    }
  });

  it("returns stable JSON shapes for core tools", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      assert.equal(tools.tools.some((tool) => tool.name === "self_review_architect_mcp"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "validate_architecture_contract"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_build_plan"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "validate_foundation_packs"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "propose_stack_pack_rules"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "discover_llms_sources"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "ingest_llms_txt"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "list_ingested_llms_sources"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "derive_stack_pack_from_llms_source"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "analyze_stack_pack_conflicts"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "interpret_implementation_intent"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "classify_ambiguity_risk"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "create_pre_edit_contract"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_implementation_against_contract"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "record_assumption"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "load_triggered_stack_guidance"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "extract_harness_memory"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "apply_harness_memory"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_memory_relevance"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "list_mcp_server_catalog"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "recommend_mcp_servers"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "create_mcp_install_plan"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_mcp_install_plan"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "apply_mcp_install_plan"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "list_skill_catalog"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "recommend_skills_for_project"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_supplied_skills"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_mcp_config_security"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "run_v3_eval_harness"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "score_agent_artifacts"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "promote_stack_pack_to_files"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "list_client_integration_recipes"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_agent_final_response"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "review_agent_session"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "audit_work_gate_completeness"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "create_work_gate_sequence_receipt"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "audit_hosted_tool_policy"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "score_stack_packs"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "stack_pack_coverage_matrix"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "scan_mcp_config_files"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "get_v10_productization_blueprint"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "create_v10_implementation_slice_plan"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "plan_primer_dashboard"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "validate_v10_productization_boundary"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "run_v10_eval_harness"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "build_quality_requirements_profile"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "evaluate_repo_plan_quality"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "audit_generated_repo_quality"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "suggest_quality_followup_questions"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "run_repo_quality_eval_scenarios"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "normalize_foundry_evidence"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "derive_repo_constitution"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "derive_local_repo_constitution"), true);

      const grilled = await callJson(client, "grill_me", {
        brief: cleanMcpServerFixture.brief,
        stackPackIds: ["mcp-server"],
        includeContract: true,
        includeArtifacts: true
      });
      assert.equal(grilled.ready, true);
      assert.equal(grilled.contract.contractVersion, "0.2.0");
      assert.equal(Array.isArray(grilled.artifacts), true);
      assert.equal(Array.isArray(grilled.buildPlan.slices), true);

      const answerResult = await callJson(client, "grill_me", {
        brief: { idea: "A small admin app" },
        includeContract: false,
        answer: {
          field: "users",
          value: "Ops admins who need to manage customer records."
        }
      });
      assert.equal(answerResult.updatedBrief.users, "Ops admins who need to manage customer records.");

      const continueResult = await callJson(client, "continue_grill_me", {
        brief: { idea: "A small admin app" },
        includeContract: false,
        answer: {
          field: "coreFlows",
          value: ["view customers", "edit customers"]
        }
      });
      assert.equal(continueResult.updatedBrief.coreFlows.length, 2);

      const contractResult = await callJson(client, "generate_architecture_contract", {
        brief: cleanMcpServerFixture.brief,
        stackPackIds: ["mcp-server"]
      });
      assert.equal(contractResult.contract.generatedBy.tool, "architect-mcp");
      assert.match(contractResult.markdown, /Review Gate/);

      const baselineResult = await callJson(client, "create_review_baseline", {
        files: messyReactFixture.files,
        directories: messyReactFixture.directories,
        contract: generateContract(messyReactFixture.brief)
      });
      assert.equal(Array.isArray(baselineResult.baseline.findings), true);
      assert.equal(typeof baselineResult.summary.errors, "number");

      const planReview = await callJson(client, "review_proposed_file_plan", {
        plan: {
          files: [
            {
              path: "src/App.tsx",
              purpose: "Own all workflows, data access, routing, state, and UI.",
              responsibilities: ["routing", "database", "auth", "state", "ui"]
            }
          ]
        }
      });
      assert.equal(planReview.report.gate.status, "fail");

      const buildPlanReview = await callJson(client, "review_build_plan", {
        plan: {
          ...grilled.buildPlan,
          slices: grilled.buildPlan.slices.filter((slice: { id: string }) => slice.id !== "agent-harness")
        },
        allowedChecks: cleanMcpServerFixture.brief.verification
      });
      assert.equal(buildPlanReview.report.gate.status, "fail");

      const findings = reviewFileSummaries(messyReactFixture.files, generateContract(messyReactFixture.brief), 300, messyReactFixture.directories);
      const reviewResult = await callJson(client, "review_repo_structure", {
        files: messyReactFixture.files,
        directories: messyReactFixture.directories,
        contract: generateContract(messyReactFixture.brief),
        baseline: createBaselineFromFindings(findings),
        mode: "strict"
      });
      assert.equal(reviewResult.report.summary.baselineSuppressed, findings.length);
      assert.equal(Array.isArray(reviewResult.lifecycle.baselineFindings), true);

      const auditResult = await callJson(client, "review_repo_structure", {
        files: publicRepoSmokeFixture.files,
        directories: publicRepoSmokeFixture.directories,
        mode: "audit"
      });
      assert.equal(auditResult.report.mode, "audit");
      assert.equal(auditResult.violations.some((violation: { code: string; message: string }) =>
        violation.code === "ARCH020_IMPLEMENTATION_IGNORED_PLAN" &&
        violation.message.includes("required agent harness artifacts")
      ), false);
      assert.equal(auditResult.report.gate.status === "pass" || auditResult.report.gate.status === "warn", true);

      const windowsPathReview = await callJson(client, "review_repo_structure", {
        files: [
          {
            path: "src\\app.ts",
            lines: 1
          }
        ],
        mode: "audit"
      });
      assert.equal(windowsPathReview.report.coverage.filesReviewed, 1);
      assert.equal(windowsPathReview.report.coverage.topScannedDirectories[0]?.directory, "src");

      const validationResult = await callJson(client, "validate_architecture_contract", {
        contract: generateContract(cleanMcpServerFixture.brief, ["mcp-server"])
      });
      assert.equal(validationResult.valid, true);

      const llmsSources = await callJson(client, "discover_llms_sources", {
        query: {
          priority: "high"
        }
      });
      assert.equal(llmsSources.sources.some((source: { id: string }) => source.id === "nextjs"), true);

      const ingestedLlmsSources = await callJson(client, "list_ingested_llms_sources", {});
      assert.equal(ingestedLlmsSources.sources.length >= 39, true);
      assert.equal(ingestedLlmsSources.sources.filter((source: { status: string }) => source.status === "ok").length >= 39, true);

      const derivedPack = await callJson(client, "derive_stack_pack_from_llms_source", {
        request: {
          sourceId: "hono"
        }
      });
      assert.equal(derivedPack.source.id, "hono");
      assert.equal(derivedPack.review.violations.length, 0);
      assert.equal(derivedPack.candidate.sources[0].note.includes("sha256"), true);

      const candidateResult = await callJson(client, "propose_stack_pack_rules", {
        input: {
          stackName: "Acme React Runtime",
          sourceText: "React components should keep UI, database access, and feature boundaries separate.",
          sourceLabel: "local snapshot"
        }
      });
      assert.equal(candidateResult.candidate.fileRules.length > 0, true);

      const candidateReview = await callJson(client, "review_stack_pack_candidate", {
        candidate: candidateResult.candidate
      });
      assert.equal(candidateReview.valid, true);

      const promotedCandidate = await callJson(client, "promote_stack_pack_candidate", {
        candidate: candidateResult.candidate
      });
      assert.equal(promotedCandidate.pack.id, "acme-react-runtime");

      const promotionFiles = await callJson(client, "promote_stack_pack_to_files", {
        request: {
          candidate: candidateResult.candidate,
          writeFiles: false
        }
      });
      assert.equal(promotionFiles.dryRun, true);
      assert.equal(promotionFiles.files.some((file: { path: string }) => file.path === "packs/acme-react-runtime.json"), true);

      const candidateDiff = await callJson(client, "diff_stack_pack_versions", {
        before: promotedCandidate.pack,
        after: {
          ...promotedCandidate.pack,
          fileRules: []
        }
      });
      assert.equal(candidateDiff.breaking, true);

      const conflictResult = await callJson(client, "analyze_stack_pack_conflicts", {
        stackPacks: [candidateResult.candidate, derivedPack.candidate]
      });
      assert.equal(Array.isArray(conflictResult.conflicts), true);

      const intentResult = await callJson(client, "interpret_implementation_intent", {
        input: {
          request: "fix this with best practices",
          stack: {
            frontend: "React",
            backend: "Hono"
          }
        }
      });
      assert.equal(intentResult.decision, "confirm_before_edit");
      assert.equal(Array.isArray(intentResult.triggeredGuidance), true);

      const riskResult = await callJson(client, "classify_ambiguity_risk", {
        input: {
          request: "make auth better"
        }
      });
      assert.equal(riskResult.decision, "confirm_before_edit");

      const contractHarness = await callJson(client, "create_pre_edit_contract", {
        intent: intentResult,
        likelyFiles: ["src/features/profile/ProfileCard.tsx"]
      });
      assert.equal(contractHarness.contract.mode, "guided-yolo");

      const malformedContract = await callToolRaw(client, "create_pre_edit_contract", {
        intent: {
          interpretedProblem: "missing required harness result fields"
        }
      });
      assert.equal(Boolean(malformedContract.isError), true);

      const implementationReview = await callJson(client, "review_implementation_against_contract", {
        input: {
          contract: contractHarness.contract,
          changedFiles: [{ path: "src/auth/session.ts", lines: 90 }],
          verification: [{ check: "npm test", status: "not_run" }]
        }
      });
      assert.equal(implementationReview.valid, false);

      const assumptionResult = await callJson(client, "record_assumption", {
        assumption: intentResult.assumptions[0]
      });
      assert.equal(assumptionResult.assumption.affectedArea, "implementation scope");

      const guidanceResult = await callJson(client, "load_triggered_stack_guidance", {
        input: {
          request: "use Hono best practices",
          selectedSourceIds: ["hono"]
        }
      });
      assert.equal(guidanceResult.guidance.some((guidance: { sourceId: string }) => guidance.sourceId === "hono"), true);

      const memoryResult = await callJson(client, "extract_harness_memory", {
        input: {
          request: "I prefer guided-yolo, keep memory stateless for now, and maybe use GitHub PRs later.",
          projectName: "architect-mcp"
        }
      });
      assert.equal(memoryResult.futureAdapter.storage, "none");
      assert.equal(memoryResult.proposals.length > 0, true);

      const appliedMemory = await callJson(client, "apply_harness_memory", {
        input: {
          request: "use guided-yolo memory on this harness task",
          memories: memoryResult.proposals,
          tokenBudget: 120
        }
      });
      assert.equal(appliedMemory.selected.length > 0, true);
      assert.equal(appliedMemory.tokenEstimate <= 120, true);

      const memoryReview = await callJson(client, "review_memory_relevance", {
        input: {
          request: "use guided-yolo memory on this harness task",
          memories: memoryResult.proposals
        }
      });
      assert.equal(Array.isArray(memoryReview.findings), true);

      const skillCatalog = await callJson(client, "list_skill_catalog", {});
      assert.equal(skillCatalog.skills.some((skill: { id: string }) => skill.id === "mcp-config-security"), true);

      const skillRecommendations = await callJson(client, "recommend_skills_for_project", {
        query: {
          request: "add MCP config security review and evidence before completion checks",
          limit: 3
        }
      });
      assert.equal(skillRecommendations.recommendations.length > 0, true);

      const suppliedSkillReview = await callJson(client, "review_supplied_skills", {
        input: {
          skills: [{
            id: "github-memory-pr",
            name: "GitHub Memory PR",
            category: "github",
            summary: "Patterns for storing memory proposals through GitHub PR review.",
            patterns: ["branch per session", "batched PR review"],
            recommendedWhen: ["github", "memory", "pr"],
            cautions: ["Do not store secrets."],
            source: "client-supplied"
          }]
        }
      });
      assert.equal(suppliedSkillReview.valid, true);

      const securityReview = await callJson(client, "review_mcp_config_security", {
        request: {
          config: {
            mcpServers: {
              risky: {
                command: "npx",
                args: ["-y", "some-mcp@latest", "--token", "sk-123456789012345678901234"]
              }
            }
          }
        }
      });
      assert.equal(securityReview.status, "fail");
      assert.equal(securityReview.findings.some((finding: { code: string }) => finding.code === "MCPSEC001_HARDCODED_SECRET"), true);

      const mcpCatalog = await callJson(client, "list_mcp_server_catalog", {
        query: { category: "database" }
      });
      assert.equal(mcpCatalog.servers.some((server: { id: string }) => server.id === "supabase"), true);

      const mcpRecommendation = await callJson(client, "recommend_mcp_servers", {
        request: {
          request: "Build with a database and auth."
        }
      });
      assert.equal(mcpRecommendation.status, "needs-clarification");

      const mcpInstallPlan = await callJson(client, "create_mcp_install_plan", {
        request: {
          serverId: "playwright",
          targetClient: "generic-json"
        }
      });
      assert.equal(mcpInstallPlan.status, "dry-run");

      const mcpInstallReview = await callJson(client, "review_mcp_install_plan", {
        request: {
          plan: mcpInstallPlan
        }
      });
      assert.equal(mcpInstallReview.status, "pass");

      const mcpInstallApplyDryRun = await callJson(client, "apply_mcp_install_plan", {
        request: {
          plan: mcpInstallPlan,
          targetPath: ".architect-mcp-test/mcp.json"
        }
      });
      assert.equal(mcpInstallApplyDryRun.status, "dry-run");

      const artifactScore = await callJson(client, "score_agent_artifacts", {
        request: {
          agentsMd: readFileSync("AGENTS.md", "utf8"),
          llmsTxt: readFileSync("llms.txt", "utf8")
        }
      });
      assert.equal(artifactScore.agentsMd.status === "pass" || artifactScore.agentsMd.status === "warn", true);
      assert.equal(artifactScore.llmsTxt.status === "pass" || artifactScore.llmsTxt.status === "warn", true);

      const evalReport = await callJson(client, "run_v3_eval_harness", {});
      assert.equal(evalReport.status, "pass");
      assert.equal(evalReport.summary.total >= 5, true);

      const recipes = await callJson(client, "list_client_integration_recipes", {});
      assert.equal(recipes.recipes.some((recipe: { id: string }) => recipe.id === "guided-yolo-pre-edit"), true);

      const finalReview = await callJson(client, "review_agent_final_response", {
        request: {
          response: "Changed the V3 tools. Verified with npm run typecheck, npm test, and npm run build. Assumptions: no new assumptions. Not done: no remaining requested work.",
          requiredChecks: ["npm run typecheck", "npm test", "npm run build"],
          receiptNow: "2026-05-17T22:30:00.000Z",
          verificationReceipts: [
            { command: "npm run typecheck", status: "passed", source: "ci", summary: "CI typecheck passed.", recordedAt: "2026-05-17T22:29:00.000Z" },
            { command: "npm test", status: "passed", source: "ci", summary: "CI tests passed.", recordedAt: "2026-05-17T22:29:00.000Z" },
            { command: "npm run build", status: "passed", source: "ci", summary: "CI build passed.", recordedAt: "2026-05-17T22:29:00.000Z" }
          ]
        }
      });
      assert.equal(finalReview.status, "pass");
      assert.equal(finalReview.verificationEvidence.receipts.complete, true);

      const gateAudit = await callJson(client, "audit_work_gate_completeness", {
        request: {
          now: "2026-05-17T22:00:00.000Z",
          records: [
            "grill_me",
            "create_pre_edit_contract",
            "review_build_plan",
            "review_proposed_file_plan",
            "review_repo_structure",
            "review_implementation_against_contract",
            "review_agent_final_response",
            "review_agent_session"
          ].map((gate) => ({ gate, status: "pass", recordedAt: "2026-05-17T22:00:00.000Z", runId: "tool-test" }))
        }
      });
      assert.equal(gateAudit.status, "pass");
      assert.equal(gateAudit.readOnly, true);

      const gateReceipt = await callJson(client, "create_work_gate_sequence_receipt", {
        request: {
          now: "2026-05-17T22:00:00.000Z",
          records: [
            "grill_me",
            "create_pre_edit_contract",
            "review_build_plan",
            "review_proposed_file_plan",
            "review_repo_structure",
            "review_implementation_against_contract",
            "review_agent_final_response",
            "review_agent_session"
          ].map((gate) => ({
            gate,
            status: "pass",
            recordedAt: "2026-05-17T22:00:00.000Z",
            runId: "tool-test",
            inputsPresent: true,
            evidencePresent: true,
            publicSummary: "Reviewed public-safe gate evidence."
          }))
        }
      });
      assert.equal(gateReceipt.status, "pass");
      assert.equal(gateReceipt.receipt.kind, "work_gate_sequence_receipt");
      assert.equal(gateReceipt.steps.every((step: { inputsPresent: boolean; evidencePresent: boolean }) => step.inputsPresent && step.evidencePresent), true);

      const unknownGateReceipt = await callJson(client, "create_work_gate_sequence_receipt", {
        request: {
          records: [{ gate: "not_a_gate", status: "pass", inputsPresent: true, evidencePresent: true }]
        }
      });
      assert.equal(unknownGateReceipt.status, "fail");
      assert.equal(unknownGateReceipt.classification, "unknown_gate");

      const policy = await callJson(client, "audit_hosted_tool_policy", {});
      assert.equal(policy.tools.some((tool: { name: string; policy: string }) => tool.name === "scan_mcp_config_files" && tool.policy === "local-only"), true);

      const packScores = await callJson(client, "score_stack_packs", {});
      assert.equal(packScores.packs.some((pack: { id: string }) => pack.id === "stripe"), true);

      const matrix = await callJson(client, "stack_pack_coverage_matrix", {});
      assert.equal(matrix.rows.some((row: { id: string; hasExecutableDetector: boolean }) => row.id === "hono" && row.hasExecutableDetector), true);

      const selfReview = await callJson(client, "self_review_architect_mcp", {});
      assert.equal(selfReview.grillMe.ready, true);
      assert.equal(selfReview.review.gate.status, "pass");

      const readiness = await callJson(client, "mcp_readiness_report", {});
      assert.equal(readiness.ready, true);
      assert.equal(readiness.checks.some((check: { name: string }) => check.name === "hosted safety"), true);

      const v10Blueprint = await callJson(client, "get_v10_productization_blueprint", {});
      assert.equal(v10Blueprint.routes.some((route: { path: string }) => route.path === "/v1/orgs"), true);
      assert.equal((await callJson(client, "run_v10_eval_harness", {})).status, "pass");

      const qualityEval = await callJson(client, "evaluate_repo_plan_quality", {
        request: {
          signals: {
            hasHardcodedSecrets: true
          }
        }
      });
      assert.equal(qualityEval.decision, "block");
      assert.equal((await callJson(client, "run_repo_quality_eval_scenarios", {})).status, "pass");

      const diffResult = await callJson(client, "diff_architecture_contracts", {
        before: generateContract(cleanMcpServerFixture.brief, ["mcp-server"]),
        after: {
          ...generateContract(cleanMcpServerFixture.brief, ["mcp-server"]),
          directories: [
            ...generateContract(cleanMcpServerFixture.brief, ["mcp-server"]).directories,
            { path: "src/new-required", purpose: "New required boundary.", required: true }
          ]
        }
      });
      assert.equal(diffResult.breaking, true);
      assert.equal(diffResult.changes.some((change: { kind: string }) => change.kind === "required-directory-added"), true);
    } finally {
      await close();
    }
  });

  it("rejects or reports invalid/malicious tool inputs predictably", async () => {
    const { client, close } = await connectTestClient();
    try {
      const unknownField = await callToolRaw(client, "grill_me", {
        brief: {
          ...cleanMcpServerFixture.brief,
          unexpectedField: true
        }
      });
      assert.equal(unknownField.isError, true);
      assert.match(unknownField.content[0].text, /Unrecognized key|validation/i);

      const unknownPack = await callToolRaw(client, "generate_architecture_contract", {
        brief: cleanMcpServerFixture.brief,
        stackPackIds: ["does-not-exist"]
      });
      assert.equal(unknownPack.isError, true);
      assert.equal(JSON.parse(unknownPack.content[0].text).error.code, "ARCHITECT_TOOL_ERROR");

      const traversalPack = await callToolRaw(client, "generate_architecture_contract", {
        brief: cleanMcpServerFixture.brief,
        stackPackIds: ["../packs/mcp-server"]
      });
      assert.equal(traversalPack.isError, true);
      assert.equal(JSON.parse(traversalPack.content[0].text).error.code, "ARCHITECT_TOOL_ERROR");

      const malformedFileSummary = await callToolRaw(client, "review_repo_structure", {
        files: [
          {
            lines: 10,
            imports: [42]
          }
        ]
      });
      assert.equal(malformedFileSummary.isError, true);
      assert.match(malformedFileSummary.content[0].text, /path|Expected string|validation/i);

      const acceptedWithoutReason = await callToolRaw(client, "review_repo_structure", {
        files: messyReactFixture.files,
        baseline: {
          findings: [
            {
              code: "ARCH001_OVERSIZED_FILE",
              status: "accepted"
            }
          ]
        }
      });
      assert.equal(acceptedWithoutReason.isError, true);
      assert.match(acceptedWithoutReason.content[0].text, /Accepted baseline findings require a reason|validation/i);

      const staleContract = {
        ...generateContract(cleanMcpServerFixture.brief, ["mcp-server"]),
        stackPacks: generateContract(cleanMcpServerFixture.brief, ["mcp-server"]).stackPacks.map((pack) => ({
          ...pack,
          version: "0.0.1"
        }))
      };
      const staleValidation = await callJson(client, "validate_architecture_contract", {
        contract: staleContract
      });
      assert.equal(staleValidation.valid, true);
      assert.equal(staleValidation.warnings.some((warning: string) => warning.includes("differs from available version")), true);

      const unknownPackContract = {
        ...generateContract(cleanMcpServerFixture.brief, ["mcp-server"]),
        stackPacks: [
          {
            ...generateContract(cleanMcpServerFixture.brief, ["mcp-server"]).stackPacks[0],
            id: "unknown-pack"
          }
        ]
      };
      const unknownPackValidation = await callJson(client, "validate_architecture_contract", {
        contract: unknownPackContract
      });
      assert.equal(unknownPackValidation.valid, false);
      assert.equal(unknownPackValidation.errors.some((error: string) => error.includes("Unknown stack pack")), true);
    } finally {
      await close();
    }
  });

  it("does not expose local workspace review in hosted mode", async () => {
    const { client, close } = await connectTestClient(false);
    try {
      const tools = await client.listTools();
      assert.equal(tools.tools.some((tool) => tool.name === "review_local_workspace"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "normalize_foundry_evidence"), true);
      assert.equal(tools.tools.some((tool) => tool.name === "derive_local_repo_constitution"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "scan_mcp_config_files"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "promote_stack_pack_to_files"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "apply_mcp_install_plan"), false);
      assert.equal(tools.tools.some((tool) => tool.name === "review_repo_structure"), true);
      const scanAttempt = await callToolRaw(client, "review_local_workspace", {
        rootPath: process.cwd()
      });
      assert.equal(scanAttempt.isError, true);
    } finally {
      await close();
    }
  });

  it("refuses local workspace scans outside cwd unless explicitly allowed", async () => {
    const { client, close } = await connectTestClient();
    const outsideRoot = mkdtempSync(join(tmpdir(), "architect-mcp-outside-"));
    writeFileSync(join(outsideRoot, "a.ts"), "export const ok = true;\n", "utf8");
    writeFileSync(join(outsideRoot, "z.ts"), "export const ok = true;\n", "utf8");
    try {
      const refused = await callToolRaw(client, "review_local_workspace", {
        rootPath: outsideRoot
      });
      assert.equal(refused.isError, true);
      assert.match(refused.content.find((content) => content.type === "text")?.text ?? "", /Refusing to scan/);

      const allowed = await callJson(client, "review_local_workspace", {
        rootPath: outsideRoot,
        maxFiles: 1,
        allowOutsideCwd: true
      });
      assert.equal(allowed.filesReviewed, 1);
      assert.equal(allowed.scan.truncated, true);
      assert.equal(allowed.report.coverage.scanTruncated, true);
      assert.equal(allowed.report.coverage.filesReviewed, 1);
      assert.equal(allowed.report.coverage.maxFiles, 1);
      assert.equal(allowed.report.coverage.topScannedDirectories[0]?.directory, "(root)");
      assert.equal(allowed.report.summary.coverageCaveats.length, 1);
    } finally {
      await close();
    }
  });

  it("does not let caller-controlled roots or symlinks bypass local scan confinement", async () => {
    const { client, close } = await connectTestClient();
    const outsideRoot = mkdtempSync(join(tmpdir(), "architect-mcp-symlink-target-"));
    const linkPath = join(process.cwd(), `.architect-mcp-symlink-${Date.now()}`);
    writeFileSync(join(outsideRoot, "sample.ts"), "export const ok = true;\n", "utf8");
    symlinkSync(outsideRoot, linkPath, "dir");

    try {
      const refusedViaAllowedRootPath = await callToolRaw(client, "review_local_workspace", {
        rootPath: outsideRoot,
        allowedRootPath: outsideRoot
      });
      assert.equal(refusedViaAllowedRootPath.isError, true);

      const refusedSymlink = await callToolRaw(client, "review_local_workspace", {
        rootPath: linkPath
      });
      assert.equal(refusedSymlink.isError, true);
      assert.match(refusedSymlink.content.find((content) => content.type === "text")?.text ?? "", /Refusing to scan/);
    } finally {
      rmSync(linkPath, { force: true });
      await close();
    }
  });

  it("every listed tool has policy metadata", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      for (const tool of tools.tools) {
        assert.equal(tool.inputSchema?.type, "object", `${tool.name} missing object input schema`);
        assert.equal(tool.outputSchema?.type, "object", `${tool.name} missing object output schema`);
      }
    } finally {
      await close();
    }
  });

  it("keeps public tool documentation in sync with registered tools", async () => {
    const { client, close } = await connectTestClient();
    try {
      const tools = await client.listTools();
      const readme = readFileSync("README.md", "utf8");
      const toolReference = readFileSync("docs/tool-reference.md", "utf8");
      const llms = readFileSync("llms.txt", "utf8");
      const toolNames = tools.tools.map((tool) => tool.name).sort();

      assert.match(readme, /https:\/\/tonycdr-prog\.github\.io\/architect-mcp\//);
      assert.equal(toolReference, generateToolReferenceMarkdown());
      assert.deepEqual(toolNames.filter((name) => !toolReference.includes(`\`${name}\``)), []);
      assert.deepEqual(toolNames.filter((name) => !llms.includes(name)), []);
    } finally {
      await close();
    }
  });
});

async function connectTestClient(enableLocalWorkspaceTool = true) {
  const server = createArchitectServer({ enableLocalWorkspaceTool, toolSurface: "advanced" });
  const client = new Client({ name: "architect-mcp-test-client", version: "0.1.0" });
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
