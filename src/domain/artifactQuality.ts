export type ArtifactQualityFinding = {
  code:
    | "ARTIFACT001_MISSING_SECTION"
    | "ARTIFACT002_MISSING_COMMAND"
    | "ARTIFACT003_MISSING_BOUNDARY"
    | "ARTIFACT004_MISSING_PROOF"
    | "ARTIFACT005_LLMSTXT_FORMAT";
  severity: "error" | "warning";
  message: string;
  recommendation: string;
};

export function scoreAgentInstructions(content: string) {
  const findings: ArtifactQualityFinding[] = [];
  const requiredSections = ["Project", "Setup", "Testing", "Code", "Proof"];

  for (const section of requiredSections) {
    if (!new RegExp(`^##\\s+.*${section}`, "im").test(content)) {
      findings.push({
        code: "ARTIFACT001_MISSING_SECTION",
        severity: section === "Testing" || section === "Proof" ? "error" : "warning",
        message: `Agent instructions are missing a ${section}-oriented section.`,
        recommendation: "Include concise operational sections for setup, workflow, testing, code boundaries, and proof of completion."
      });
    }
  }

  if (!/\b(?:npm|pnpm|yarn|bun|uv|cargo|go)\s+\w+/.test(content)) {
    findings.push({
      code: "ARTIFACT002_MISSING_COMMAND",
      severity: "error",
      message: "Agent instructions do not include executable setup or verification commands.",
      recommendation: "Name the exact commands agents should run, such as npm test or npm run typecheck."
    });
  }

  if (!/Do Not Create|module boundaries|boundar/i.test(content)) {
    findings.push({
      code: "ARTIFACT003_MISSING_BOUNDARY",
      severity: "error",
      message: "Agent instructions do not define implementation boundaries.",
      recommendation: "State the files, ownership rules, and architecture boundaries agents must respect."
    });
  }

  if (!/proof|verified|verification|completion/i.test(content)) {
    findings.push({
      code: "ARTIFACT004_MISSING_PROOF",
      severity: "error",
      message: "Agent instructions do not require proof before completion claims.",
      recommendation: "Require fresh verification evidence and disclosure of skipped checks."
    });
  }

  return qualityReport(findings);
}

export function scoreLlmsTxt(content: string) {
  const findings: ArtifactQualityFinding[] = [];
  if (!/^#\s+\S/m.test(content)) {
    findings.push({
      code: "ARTIFACT005_LLMSTXT_FORMAT",
      severity: "error",
      message: "llms.txt is missing the required H1 project header.",
      recommendation: "Start llms.txt with a single H1 naming the project."
    });
  }

  if (!/^>\s+.+/m.test(content)) {
    findings.push({
      code: "ARTIFACT005_LLMSTXT_FORMAT",
      severity: "warning",
      message: "llms.txt is missing a blockquote summary.",
      recommendation: "Add a short blockquote summary explaining the repository purpose."
    });
  }

  if (!/^##\s+/m.test(content) || !/\[[^\]]+\]\([^)]+\):/.test(content)) {
    findings.push({
      code: "ARTIFACT005_LLMSTXT_FORMAT",
      severity: "error",
      message: "llms.txt does not expose navigable H2 file-list sections with described links.",
      recommendation: "Use H2 sections and links in the form [label](path): description."
    });
  }

  if (!/README\.md/.test(content) || !/docs\//.test(content)) {
    findings.push({
      code: "ARTIFACT001_MISSING_SECTION",
      severity: "warning",
      message: "llms.txt does not point to both README and docs content.",
      recommendation: "Include primary human docs plus deeper architecture/specification docs."
    });
  }

  if (!/tool|workflow|finding|contract/i.test(content)) {
    findings.push({
      code: "ARTIFACT003_MISSING_BOUNDARY",
      severity: "warning",
      message: "llms.txt does not clearly identify tool/workflow contracts.",
      recommendation: "Make MCP tools, workflows, finding codes, and contract entry points discoverable."
    });
  }

  return qualityReport(findings);
}

function qualityReport(findings: ArtifactQualityFinding[]) {
  const errors = findings.filter((finding) => finding.severity === "error").length;
  const warnings = findings.filter((finding) => finding.severity === "warning").length;
  const score = Math.max(0, 100 - errors * 25 - warnings * 10);
  return {
    score,
    status: errors > 0 ? "fail" : warnings > 0 ? "warn" : "pass",
    summary: { errors, warnings },
    findings
  };
}
