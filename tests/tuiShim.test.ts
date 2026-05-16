import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const shim = require("../bin/architect-mcp-tui.cjs") as {
  archTag: () => string;
  ensureCachedReleaseBinary: (options: Record<string, unknown>) => Promise<string>;
  findLocalBinary: (candidates: string[]) => string | undefined;
  platformTag: () => string;
};

const binaryName = process.platform === "win32" ? "architect-mcp-tui.exe" : "architect-mcp-tui";

describe("architect-mcp-tui npm shim", () => {
  it("prefers an executable local binary when present", () => {
    const temp = mkdtempSync(join(tmpdir(), "architect-tui-shim-"));
    const binary = join(temp, binaryName);
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);

    assert.equal(shim.findLocalBinary([join(temp, "missing"), binary]), binary);
  });

  it("returns a cached release binary without network fetches", async () => {
    const cacheRoot = mkdtempSync(join(tmpdir(), "architect-tui-cache-"));
    const version = "9.9.9";
    const platform = shim.platformTag();
    const arch = shim.archTag();
    const cacheDir = join(cacheRoot, version, `${platform}-${arch}`);
    mkdirSync(cacheDir, { recursive: true });
    const binary = join(cacheDir, binaryName);
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);

    const resolved = await shim.ensureCachedReleaseBinary({
      cacheRoot,
      version,
      platform,
      arch,
      downloadText: async () => {
        throw new Error("network should not be used");
      }
    });
    assert.equal(resolved, binary);
  });

  it("fails closed when no local or downloadable release binary is available", async () => {
    await assert.rejects(
      () =>
        shim.ensureCachedReleaseBinary({
          cacheRoot: mkdtempSync(join(tmpdir(), "architect-tui-missing-")),
          version: "0.0.0-missing",
          platform: shim.platformTag(),
          arch: shim.archTag(),
          downloadText: async () => {
            throw new Error("not found");
          }
        }),
      /Build locally with "npm run tui:build"/
    );
  });

  it("rejects release archives with checksum mismatches", async () => {
    await assert.rejects(
      () =>
        shim.ensureCachedReleaseBinary({
          cacheRoot: mkdtempSync(join(tmpdir(), "architect-tui-bad-sha-")),
          version: "0.0.0-bad-sha",
          platform: shim.platformTag(),
          arch: shim.archTag(),
          downloadText: async () => "0000",
          downloadFile: async (_url: string, destination: string) => {
            writeFileSync(destination, "archive");
          },
          extractArchive: () => {
            throw new Error("extract should not run");
          }
        }),
      /checksum mismatch/
    );
  });
});
