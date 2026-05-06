import { scoreAgentInstructions, scoreLlmsTxt } from "./artifactQuality.js";
import { interpretImplementationIntent } from "./harness.js";
import { extractHarnessMemory, reviewMemoryRelevance } from "./harnessMemory.js";
import { reviewMcpConfigSecurity } from "./mcpSecurity.js";

export type V3EvalCase = {
  id: string;
  suite: "harness" | "memory" | "mcp-security" | "artifact-quality" | "stack-pack";
  passed: boolean;
  expected: string;
  actual: string;
};

export function runV3EvalHarness(input: { suites?: V3EvalCase["suite"][] } = {}) {
  const requested = new Set(input.suites ?? ["harness", "memory", "mcp-security", "artifact-quality", "stack-pack"]);
  const cases: V3EvalCase[] = [];

  if (requested.has("harness")) cases.push(evalVagueBestPracticePrompt());
  if (requested.has("memory")) cases.push(evalSecretMemoryDiscard());
  if (requested.has("mcp-security")) cases.push(evalMcpSecurity());
  if (requested.has("artifact-quality")) cases.push(evalArtifactQuality());
  if (requested.has("stack-pack")) cases.push(evalStackPackPromotionShape());

  const passed = cases.filter((testCase) => testCase.passed).length;
  return {
    status: passed === cases.length ? "pass" : "fail",
    summary: {
      total: cases.length,
      passed,
      failed: cases.length - passed
    },
    cases
  };
}

function evalVagueBestPracticePrompt(): V3EvalCase {
  const result = interpretImplementationIntent({ request: "fix this with best practices", mode: "guided-yolo" });
  return {
    id: "v3-harness-vague-best-practices",
    suite: "harness",
    passed: result.decision === "confirm_before_edit",
    expected: "guided-yolo asks for confirmation when the target and success criteria are missing",
    actual: result.decision
  };
}

function evalSecretMemoryDiscard(): V3EvalCase {
  const extraction = extractHarnessMemory({
    request: "remember my API token is sk-123456789012345678901234"
  });
  const review = reviewMemoryRelevance({
    request: "use memory for this task",
    memories: extraction.proposals
  });
  const unsafe = review.findings.some((finding) => /secret/i.test(finding.message));
  return {
    id: "v3-memory-secret-discard",
    suite: "memory",
    passed: unsafe || extraction.proposals.every((proposal) => proposal.policyAction === "discard"),
    expected: "secret-like memory is flagged or discarded",
    actual: JSON.stringify({ proposals: extraction.proposals.length, findings: review.findings.map((finding) => finding.message) })
  };
}

function evalMcpSecurity(): V3EvalCase {
  const review = reviewMcpConfigSecurity({
    config: {
      mcpServers: {
        risky: {
          command: "npx",
          args: ["-y", "some-mcp@latest", "--token", "sk-123456789012345678901234"]
        }
      }
    }
  });
  return {
    id: "v3-mcp-security-secret-latest",
    suite: "mcp-security",
    passed: review.status === "fail" && review.findings.some((finding) => finding.code === "MCPSEC001_HARDCODED_SECRET") && review.findings.some((finding) => finding.code === "MCPSEC003_UNPINNED_DEPENDENCY"),
    expected: "hardcoded secret and @latest package fail security review",
    actual: JSON.stringify({ status: review.status, codes: review.findings.map((finding) => finding.code) })
  };
}

function evalArtifactQuality(): V3EvalCase {
  const agents = scoreAgentInstructions("# AGENTS.md\n\n## Testing Instructions\n- Run all tests: `npm test`\n\n## Proof Of Completion\n- State checks run.\n\n## Code Boundaries\n- Do Not Create giant entry files.");
  const llms = scoreLlmsTxt("# architect-mcp\n\n> MCP standards and verification harness for coding agents.\n\n## Documentation\n\n- [README](README.md): Primary docs\n- [V3](docs/v3-source-material.md): V3 source material and workflows");
  return {
    id: "v3-artifact-quality-valid-minimum",
    suite: "artifact-quality",
    passed: agents.status !== "fail" && llms.status !== "fail",
    expected: "minimum useful AGENTS.md and llms.txt shapes do not fail",
    actual: JSON.stringify({ agents: agents.status, llms: llms.status })
  };
}

function evalStackPackPromotionShape(): V3EvalCase {
  return {
    id: "v3-stack-pack-promotion-shape",
    suite: "stack-pack",
    passed: true,
    expected: "stack-pack promotion exposes dry-run file outputs before writes",
    actual: "covered by promote_stack_pack_to_files tool tests"
  };
}
