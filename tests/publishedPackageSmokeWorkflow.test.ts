import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

describe("published package TUI smoke workflow", () => {
  it("smokes the published npm package on hosted Ubuntu and Windows only", () => {
    const workflow = readFileSync(".github/workflows/tui-published-package-smoke.yml", "utf8");
    const terminalQa = readFileSync("docs/terminal-qa.md", "utf8");
    const tuiLiveQa = readFileSync("docs/tui-live-qa.md", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /ubuntu-latest/);
    assert.match(workflow, /windows-latest/);
    assert.doesNotMatch(workflow, /macos-14/);
    assert.match(workflow, /node-version: "22"/);
    assert.match(workflow, /npm install --prefix "\$smoke_dir" @tonycdr-prog\/architect-mcp@latest/);
    assert.match(workflow, /node_modules", "@tonycdr-prog", "architect-mcp", "dist", "index\.js"/);
    assert.match(workflow, /path\.join\(smokeDir, "tui\.toml"\)/);
    assert.match(workflow, /npm exec --prefix "\$smoke_dir" -- architect-mcp-tui --help/);
    assert.match(workflow, /architect-mcp-tui --config "\$smoke_dir\/tui\.toml" config adapters --json/);
    assert.match(workflow, /architect-mcp-tui --config "\$smoke_dir\/tui\.toml" run/);
    assert.match(workflow, /--jsonl/);
    assert.match(workflow, /approval_required/);
    assert.match(workflow, /adapter_started/);
    assert.doesNotMatch(workflow, /npm ci/);
    assert.doesNotMatch(workflow, /actions\/checkout/);
    assert.doesNotMatch(workflow, /architect-mcp-tui\s*$/m);
    assert.match(terminalQa, /published-package smoke workflow/);
    assert.match(terminalQa, /hosted non-interactive baseline evidence only/);
    assert.match(tuiLiveQa, /TUI published package smoke workflow/);
    assert.match(tuiLiveQa, /does not replace #136 manual terminal QA/);
  });
});
