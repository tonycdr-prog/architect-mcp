import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promoteStackPackCandidateToFiles, proposeStackPackRules } from "../src/domain/stackPackWorkflow.js";

describe("promoteStackPackCandidateToFiles", () => {
  it("returns dry-run pack and manifest outputs with readiness metadata", () => {
    const candidate = proposeStackPackRules({
      stackName: "Acme Hono Runtime",
      sourceText: "Hono server route handlers should stay thin and call service modules for workflow orchestration.",
      sourceLabel: "local reviewed source"
    });

    const result = promoteStackPackCandidateToFiles(candidate);

    assert.equal(result.dryRun, true);
    assert.equal(result.readiness.status, "ready");
    assert.equal(result.readiness.existingPack, false);
    assert.equal(result.files.some((file) => file.path === "packs/acme-hono-runtime.json"), true);
    assert.equal(result.files.some((file) => file.path === "packs/manifest.json"), true);
    assert.equal(result.readiness.manifestChanges.some((change) => change.id === "acme-hono-runtime"), true);
  });

  it("blocks existing packs unless overwrite is explicit and supports version bumps", () => {
    const candidate = proposeStackPackRules({
      stackName: "React",
      sourceText: "React components should keep UI, database access, and feature boundaries separate.",
      sourceLabel: "local reviewed source"
    });

    const blocked = promoteStackPackCandidateToFiles(candidate);
    const bumped = promoteStackPackCandidateToFiles(candidate, {
      allowOverwrite: true,
      bump: "patch"
    });

    assert.equal(blocked.readiness.status, "blocked");
    assert.equal(blocked.readiness.existingPack, true);
    assert.equal(bumped.readiness.status, "ready");
    assert.equal(bumped.pack.version, "0.1.1");
    assert.equal(bumped.readiness.versionChanged, true);
  });

  it("supports candidate to dry-run to temp write lifecycle", () => {
    const root = mkdtempSync(join(tmpdir(), "architect-pack-lifecycle-"));
    const packDirectory = join(root, "packs");
    mkdirSync(packDirectory);
    writeFileSync(join(packDirectory, "manifest.json"), JSON.stringify({ generatedBy: "architect-mcp", entries: [] }, null, 2), "utf8");
    const candidate = proposeStackPackRules({
      stackName: "Acme Zod Runtime",
      sourceText: "Zod validation schemas should stay near transport-safe boundaries and outside giant UI components.",
      sourceLabel: "local reviewed source"
    });

    const dryRun = promoteStackPackCandidateToFiles(candidate, { packDirectory });
    const written = promoteStackPackCandidateToFiles(candidate, { packDirectory, writeFiles: true });
    const manifest = JSON.parse(readFileSync(join(packDirectory, "manifest.json"), "utf8")) as { entries: Array<{ id: string }> };

    assert.equal(dryRun.dryRun, true);
    assert.equal(dryRun.files.some((file) => file.path.endsWith("/packs/acme-zod-runtime.json")), true);
    assert.equal(dryRun.files.some((file) => file.path.endsWith("/packs/manifest.json")), true);
    assert.equal(dryRun.warnings.some((warning) => warning.includes(`${packDirectory}/acme-zod-runtime.json`) && warning.includes(`${packDirectory}/manifest.json`)), true);
    assert.equal(written.dryRun, false);
    assert.equal(manifest.entries.some((entry) => entry.id === "acme-zod-runtime"), true);
    assert.equal(JSON.parse(readFileSync(join(packDirectory, "acme-zod-runtime.json"), "utf8")).id, "acme-zod-runtime");
  });
});
