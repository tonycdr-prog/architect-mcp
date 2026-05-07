import { createFinding } from "./findingMetadata.js";
import type { ProposedFilePlan, ReviewViolation } from "./types.js";

const APP_FILE_PATTERN = /(^|\/)(App|app|index|server|main)\.(tsx|ts|jsx|js)$/;

export function reviewProposedFilePlan(plan: ProposedFilePlan): ReviewViolation[] {
  const findings: ReviewViolation[] = [];
  const files = plan.files;
  const paths = files.map((file) => file.path);

  for (const file of files) {
    const responsibilities = file.responsibilities ?? [];
    const responsibilityText = `${file.purpose} ${responsibilities.join(" ")}`;
    const highRiskAppEntry = APP_FILE_PATTERN.test(file.path) && highRiskResponsibilityCount(responsibilityText) >= 4;
    if (responsibilities.length > 3 || highRiskAppEntry || (APP_FILE_PATTERN.test(file.path) && /workflow|database|auth|routing|state|ui/i.test(responsibilityText))) {
      findings.push(createFinding({
        code: "ARCH015_PLAN_MONOLITH_RISK",
        severity: highRiskAppEntry ? "error" : "warning",
        confidence: highRiskAppEntry ? "high" : undefined,
        path: file.path,
        message: "Proposed file plan concentrates too many responsibilities in one file.",
        recommendation: "Split app entry, routes/screens, services, data access, state, and presentation into named modules."
      }));
    }
  }

  if (paths.some((path) => /\.(tsx|jsx)$/.test(path)) && !paths.some((path) => /features?|components?|screens?|pages?/.test(path))) {
    findings.push(createFinding({
      code: "ARCH016_PLAN_MISSING_BOUNDARY",
      severity: "error",
      message: "Proposed frontend plan has UI files but no feature/component/screen boundary.",
      recommendation: "Add feature-owned UI folders before implementing screens or routes."
    }));
  }

  if (paths.some((path) => /db|schema|migration/.test(path)) && !paths.some((path) => /server|service|repository|queries|data-access/.test(path))) {
    findings.push(createFinding({
      code: "ARCH016_PLAN_MISSING_BOUNDARY",
      severity: "error",
      message: "Proposed data plan lacks a server/database ownership boundary.",
      recommendation: "Add server-owned repositories/query modules before UI consumes database data."
    }));
  }

  if (!paths.includes("AGENTS.md") || !paths.includes("docs/architecture-contract.md")) {
    findings.push(createFinding({
      code: "ARCH017_PLAN_MISSING_HARNESS",
      severity: "error",
      message: "Proposed plan does not include required agent harness artifacts.",
      recommendation: "Generate AGENTS.md and docs/architecture-contract.md before implementation files."
    }));
  }

  return findings;
}

function highRiskResponsibilityCount(text: string): number {
  return ["workflow", "database", "auth", "routing", "state", "ui"].filter((term) => responsibilityTermPattern(term).test(text)).length;
}

function responsibilityTermPattern(term: string): RegExp {
  return new RegExp(`(^|[^a-z0-9])${term}([^a-z0-9]|$)`, "i");
}
