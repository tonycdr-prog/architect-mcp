import { foundryEvalReviewReport } from "./foundryEvalCorpusReviewFixture.js";
import type { FoundryEvalCorpusCase } from "./foundryEvalCorpusTypes.js";
import type { RepoConstitutionArtifact, RepoConstitutionPullRequest } from "./repoConstitutionTypes.js";

const standardArtifacts: RepoConstitutionArtifact[] = [
  { path: "AGENTS.md", content: "Use repo-local checks and do not mutate external repositories without maintainer approval." },
  { path: "README.md", content: "# Fixture Repo\n\nInstall, test, and contribute through the repository checks." },
  { path: ".github/workflows/ci.yml", content: "name: CI\njobs:\n  test:\n    steps:\n      - run: npm test\n" },
  { path: ".github/PULL_REQUEST_TEMPLATE.md", content: "## Summary\n\n## Verification\n\n## Linked issues\n\n## Release notes\n" },
  { path: "package.json", content: "{\"scripts\":{\"test\":\"npm test\"}}" }
];

const standardRecentPullRequests: RepoConstitutionPullRequest[] = [
  { number: 120, title: "Tighten fixture routing", author: "maintainer", authorAssociation: "MEMBER", merged: true, body: "## Summary\nUpdated routing.\n\n## Verification\n- npm test\n" },
  { number: 121, title: "Document checks", author: "maintainer", authorAssociation: "OWNER", merged: true, body: "## Summary\nDocs only.\n\n## Verification\n- docs build\n" }
];

export const foundryEvalCorpusFixtureCases: FoundryEvalCorpusCase[] = [
  {
    id: "large-nextjs-docs-generated-noise",
    repo: { slug: "vercel/next.js", size: "large", type: "web_framework", language: "TypeScript" },
    expectedRoutes: ["exception"],
    expectedNoisePatterns: ["docs-example client/server warning", "generated data-file oversized warning", "large-repo scan truncation caveat"],
    regressionIssues: [310, 311, 313, 317],
    artifacts: standardArtifacts,
    recentPullRequests: standardRecentPullRequests,
    evidence: {
      reviewReports: [foundryEvalReviewReport({
        scanTruncated: true,
        filesReviewed: 5000,
        maxFiles: 5000,
        caveats: ["Large repo scan stopped at the configured file cap."],
        violations: [
          {
            code: "ARCH003_CLIENT_SERVER_LEAK",
            severity: "warning",
            confidence: "high",
            path: "docs/app/building-your-application/example.mdx",
            message: "Documentation example includes a client/server environment reference.",
            recommendation: "Treat docs examples as suppression candidates before actionability routing."
          },
          {
            code: "ARCH001_OVERSIZED_FILE",
            severity: "warning",
            confidence: "high",
            path: "dist/generated/routes-manifest.json",
            message: "Generated manifest exceeds the normal source-file size budget.",
            recommendation: "Do not route generated data-file size warnings as maintainer PR previews."
          }
        ]
      })],
      verification: [passed("fixture docs smoke")]
    }
  },
  {
    id: "small-library-pr-preview",
    repo: { slug: "expressjs/express", size: "small", type: "node_library", language: "JavaScript" },
    expectedRoutes: ["pr_preview"],
    expectedNoisePatterns: ["repo PR template should outrank recent accepted PR style", "high-confidence low-blast maintainer-owned finding"],
    regressionIssues: [314, 316],
    artifacts: standardArtifacts,
    recentPullRequests: standardRecentPullRequests,
    evidence: {
      findings: [{
        code: "ARCH008_ENV_SCATTER",
        severity: "error",
        confidence: "high",
        path: "lib/router/env.ts",
        message: "Environment access is scattered across a maintainer-owned module.",
        recommendation: "Centralize the environment lookup and verify with the package test command."
      }],
      verification: [passed("npm test")]
    }
  },
  {
    id: "python-framework-issue-preview",
    repo: { slug: "pallets/flask", size: "small", type: "python_framework", language: "Python" },
    expectedRoutes: ["architect_issue"],
    expectedNoisePatterns: ["missing agent instructions should not be an external-repo hard gate", "missing verification should prefer issue preview over auto PR"],
    regressionIssues: [318, 319],
    artifacts: [
      { path: "README.rst", content: "Flask fixture with public contributor guidance." },
      { path: "pyproject.toml", content: "[project]\nname = \"fixture-flask\"\n" }
    ],
    recentPullRequests: [{ number: 85, title: "Update supported Python docs", author: "maintainer", authorAssociation: "MEMBER", merged: true, body: "## What changed\nDocs update.\n\n## Tests\n- tox\n" }],
    evidence: {
      externalFindings: [{
        toolName: "repo-profile-check",
        ruleId: "PYTHON_VERSION_DOC_DRIFT",
        severity: "error",
        confidence: "medium",
        message: "Contributor metadata appears inconsistent with the supported runtime policy.",
        recommendation: "Open an architect issue preview and ask the maintainer for the expected Python support policy."
      }]
    },
    verificationHints: ["tox"]
  },
  {
    id: "rust-cli-entrypoint-noise",
    repo: { slug: "BurntSushi/ripgrep", size: "small", type: "rust_cli", language: "Rust" },
    expectedRoutes: ["exception"],
    expectedNoisePatterns: ["conventional CLI entrypoint should not be hygiene drift by default", "Rust repo profile should influence source-layout expectations"],
    regressionIssues: [315, 319],
    artifacts: [...standardArtifacts.filter((artifact) => artifact.path !== "package.json"), { path: "Cargo.toml", content: "[package]\nname = \"fixture-ripgrep\"\n" }],
    recentPullRequests: standardRecentPullRequests,
    evidence: {
      findings: [{
        code: "ARCH026_REPO_HYGIENE",
        severity: "warning",
        confidence: "high",
        path: "src/main.rs",
        message: "The CLI entrypoint has broad wiring responsibilities.",
        recommendation: "Treat conventional package entrypoints as suppression candidates before routing."
      }],
      verification: [passed("cargo test")]
    }
  },
  {
    id: "tiny-package-noop-and-human",
    repo: { slug: "chalk/chalk", size: "tiny", type: "node_library", language: "JavaScript" },
    expectedRoutes: ["no_op", "ask_human"],
    expectedNoisePatterns: ["low-value style-only signal", "security-sensitive raw payload requires human disclosure review", "TSX and env-like text must be context and disclosure aware"],
    regressionIssues: [312, 316],
    artifacts: standardArtifacts,
    recentPullRequests: standardRecentPullRequests,
    evidence: {
      externalFindings: [
        {
          toolName: "style-audit",
          ruleId: "OPTIONAL_COMMENT_STYLE",
          severity: "info",
          confidence: "high",
          message: "A style-only comment could be slightly clearer.",
          recommendation: "Record a no-op unless the maintainer already requested this cleanup."
        },
        {
          toolName: "security-probe",
          ruleId: "SECRET_IN_ENV_SAMPLE",
          severity: "error",
          confidence: "high",
          path: "/Users/example/private/chalk/.env",
          message: "Security-sensitive evidence includes token ghp_secretcorp123456 and a secret internal path.",
          recommendation: "Ask a human maintainer to decide safe disclosure boundaries before any public preview.",
          rawPayload: { marker: "RAW_PRIVATE_PAYLOAD", detail: "private stack trace with sk-secretcorpus123456" },
          securitySensitive: true
        }
      ],
      verification: [passed("npm test")]
    }
  }
];

function passed(check: string) {
  return {
    check,
    status: "passed" as const,
    summary: "Fixture verification passed.",
    recordedAt: "2026-05-20T00:00:00Z"
  };
}
