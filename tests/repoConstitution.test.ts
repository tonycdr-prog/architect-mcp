import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { deriveRepoConstitution } from "../src/domain/repoConstitution.js";

describe("deriveRepoConstitution", () => {
  it("derives hard repo signals with provenance without raw content", () => {
    const constitution = deriveRepoConstitution({
      files: [
        { path: "AGENTS.md", lines: 20 },
        { path: "README.md", lines: 80 },
        { path: ".github/pull_request_template.md", lines: 30 },
        { path: ".github/workflows/ci.yml", lines: 40 },
        { path: ".github/labeler.yml", lines: 20 },
        { path: "package.json", lines: 30 },
        { path: "src/index.ts", lines: 20 },
        { path: "CHANGELOG.md", lines: 10 }
      ],
      artifacts: [
        {
          path: ".github/pull_request_template.md",
          content: "## Summary\n\n## Verification\n- [ ] npm test\n\n## Release notes\n"
        },
        {
          path: ".github/workflows/ci.yml",
          content: "name: CI\non:\n  pull_request:\n  push:\n"
        },
        {
          path: "package.json",
          content: JSON.stringify({ name: "demo", version: "1.2.3", scripts: { test: "node --test", build: "tsc" } })
        },
        {
          path: "AGENTS.md",
          content: "Do the right thing."
        }
      ]
    });

    assert.equal(constitution.publicSafety.rawContentIncluded, false);
    assert.equal(constitution.publicSafety.mutationAllowed, false);
    assert.deepEqual(constitution.instructions.agentInstructionPaths, ["AGENTS.md"]);
    assert.equal(constitution.pullRequests.templates[0].mentionsVerification, true);
    assert.equal(constitution.pullRequests.templates[0].mentionsReleaseNotes, true);
    assert.deepEqual(constitution.ci.workflows[0].triggers, ["pull_request", "push"]);
    assert.deepEqual(constitution.ci.labelerConfigPaths, [".github/labeler.yml"]);
    assert.equal(constitution.packageMetadata[0].name, "demo");
    assert.deepEqual(constitution.packageMetadata[0].scripts, ["build", "test"]);
    assert.deepEqual(constitution.release.changelogPaths, ["CHANGELOG.md"]);
    assert.equal(constitution.provenance.some((entry) => entry.path === ".github/pull_request_template.md"), true);
    assert.equal(JSON.stringify(constitution).includes("Do the right thing"), false);
  });

  it("uses accepted PR style as advisory fallback when templates are missing", () => {
    const constitution = deriveRepoConstitution({
      files: [
        { path: "README.md", lines: 80 },
        { path: ".github/workflows/ci.yml", lines: 20 },
        { path: "package.json", lines: 30 }
      ],
      recentPullRequests: [
        {
          number: 41,
          author: "maintainer",
          authorAssociation: "OWNER",
          merged: true,
          body: "## What?\n\n## Why?\n\n## Test Plan\n- [x] npm test\n"
        },
        {
          number: 43,
          author: "external-contributor",
          authorAssociation: "CONTRIBUTOR",
          merged: true,
          body: "## What?\n\n## Test Plan\n"
        },
        {
          number: 42,
          author: "dependabot[bot]",
          merged: true,
          body: "## Dependencies\n"
        },
        {
          number: 44,
          author: "open-pr-author",
          body: "## Draft Shape\n"
        }
      ]
    });

    assert.equal(constitution.pullRequests.recentStyle.advisory, true);
    assert.equal(constitution.pullRequests.recentStyle.sampleSize, 3);
    assert.equal(constitution.pullRequests.recentStyle.acceptedSamples, 2);
    assert.equal(constitution.pullRequests.recentStyle.maintainerAuthoredSamples, 1);
    assert.equal(constitution.pullRequests.recentStyle.botSamplesIgnored, 1);
    assert.equal(constitution.pullRequests.recentStyle.nonMergedOrUnknownSamplesIgnored, 1);
    assert.deepEqual(constitution.pullRequests.recentStyle.commonHeadings.map((entry) => entry.heading), ["Test Plan", "What?", "Why?"]);
    assert.equal(constitution.provenance.some((entry) => /Accepted merged PR #43/.test(entry.detail)), true);
    assert.equal(constitution.provenance.some((entry) => /Accepted merged PR #44/.test(entry.detail)), false);
    assert.equal(constitution.findings.some((finding) => finding.code === "MISSING_PR_TEMPLATE"), true);
    assert.match(constitution.pullRequests.precedence.join("\n"), /advisory fallback/i);
  });

  it("discovers root and docs PR templates plus scalar workflow triggers", () => {
    const constitution = deriveRepoConstitution({
      files: [
        { path: "pull_request_template.md", lines: 5 },
        { path: "docs/PULL_REQUEST_TEMPLATE/bug.md", lines: 5 },
        { path: ".github/workflows/push.yml", lines: 2 }
      ],
      artifacts: [
        { path: "pull_request_template.md", content: "## Summary\n" },
        { path: "docs/PULL_REQUEST_TEMPLATE/bug.md", content: "## Bug\n" },
        { path: ".github/workflows/push.yml", content: "name: Push\non: push\n" }
      ]
    });

    assert.deepEqual(constitution.pullRequests.templates.map((template) => template.path), [
      "docs/PULL_REQUEST_TEMPLATE/bug.md",
      "pull_request_template.md"
    ]);
    assert.deepEqual(constitution.ci.workflows[0].triggers, ["push"]);
  });

  it("summarizes polyglot repos before single package manager shapes", () => {
    const constitution = deriveRepoConstitution({
      files: [
        { path: "package.json", lines: 5 },
        { path: "pyproject.toml", lines: 5 },
        { path: "src/index.ts", lines: 3 },
        { path: "service/app.py", lines: 3 }
      ],
      artifacts: [
        { path: "package.json", content: JSON.stringify({ name: "web" }) },
        { path: "pyproject.toml", content: "name = \"api\"\nversion = \"0.1.0\"\n" }
      ]
    });

    assert.equal(constitution.summary.repoShape, "polyglot");
    assert.deepEqual(constitution.summary.packageManagers, ["npm", "python"]);
  });

  it("warns when recent accepted style suggests a possibly stale template", () => {
    const constitution = deriveRepoConstitution({
      files: [
        { path: ".github/pull_request_template.md", lines: 20 },
        { path: ".github/workflows/ci.yml", lines: 20 }
      ],
      artifacts: [
        {
          path: ".github/pull_request_template.md",
          content: "## Summary\n\n## Verification\n"
        }
      ],
      recentPullRequests: [
        {
          number: 7,
          author: "maintainer",
          authorAssociation: "MEMBER",
          merged: true,
          body: "## What?\n\n## Why?\n"
        }
      ]
    });

    assert.equal(constitution.findings.some((finding) => finding.code === "RECENT_PR_STYLE_DIVERGES"), true);
    assert.match(constitution.findings.map((finding) => finding.message).join("\n"), /may be stale/);
    assert.match(constitution.findings.map((finding) => finding.recommendation).join("\n"), /template as authoritative/i);
  });

  it("treats hidden-comment templates as sparse while preserving recent style as advisory", () => {
    const constitution = deriveRepoConstitution({
      files: [
        { path: ".github/PULL_REQUEST_TEMPLATE/default.md", lines: 10 },
        { path: "src/lib.rs", lines: 100 }
      ],
      artifacts: [
        {
          path: ".github/PULL_REQUEST_TEMPLATE/default.md",
          content: "<!-- Fill out the required maintainer checklist. -->"
        },
        {
          path: "Cargo.toml",
          content: "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"
        }
      ],
      recentPullRequests: [
        {
          number: 9,
          author: "maintainer",
          authorAssociation: "COLLABORATOR",
          merged: true,
          body: "## Summary\n\n## Testing\n"
        }
      ]
    });

    assert.equal(constitution.pullRequests.templates[0].hiddenCommentOnly, true);
    assert.equal(constitution.findings.some((finding) => finding.code === "HIDDEN_OR_SPARSE_PR_TEMPLATE"), true);
    assert.equal(constitution.pullRequests.recentStyle.acceptedSamples, 1);
    assert.equal(constitution.pullRequests.recentStyle.maintainerAuthoredSamples, 1);
  });
});
