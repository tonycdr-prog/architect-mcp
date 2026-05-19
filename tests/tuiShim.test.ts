import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";
import { EventEmitter } from "node:events";
import https from "node:https";

const require = createRequire(import.meta.url);
const shim = require("../bin/architect-mcp-tui.cjs") as {
  archTag: () => string;
  downloadFile: (url: string, destination: string) => Promise<void>;
  downloadText: (url: string) => Promise<string>;
  ensureCachedReleaseBinary: (options: Record<string, unknown>) => Promise<string>;
  findLocalBinary: (
    candidates: string[],
    options?: {
      requiredHelpCommands?: string[];
      runHelp?: (binaryPath: string) => { ok: boolean; output: string };
    }
  ) => string | undefined;
  localBinarySupportsCommands: (
    binaryPath: string,
    requiredHelpCommands?: string[],
    options?: {
      runHelp?: (binaryPath: string) => { ok: boolean; output: string };
    }
  ) => boolean;
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

  it("skips stale local binaries that do not expose required commands", () => {
    const temp = mkdtempSync(join(tmpdir(), "architect-tui-local-"));
    const stale = join(temp, `stale-${binaryName}`);
    const current = join(temp, `current-${binaryName}`);
    writeFileSync(stale, "stale\n");
    writeFileSync(current, "current\n");
    chmodSync(stale, 0o755);
    chmodSync(current, 0o755);

    const resolved = shim.findLocalBinary([stale, current], {
      requiredHelpCommands: ["smoke", "walkthrough", "promotion-smoke", "foundry-smoke", "governance-audit"],
      runHelp: (binaryPath) => ({
        ok: true,
        output:
          binaryPath === stale
            ? "Commands:\n  run\n  config\n  smoke\n"
            : "Commands:\n  run\n  config\n  smoke\n  walkthrough\n  promotion-smoke\n  foundry-smoke\n  governance-audit\n",
      }),
    });

    assert.equal(resolved, current);
  });

  it("treats help failures as an unusable local binary when commands are required", () => {
    const temp = mkdtempSync(join(tmpdir(), "architect-tui-help-fail-"));
    const binary = join(temp, binaryName);
    writeFileSync(binary, "binary\n");
    chmodSync(binary, 0o755);

    assert.equal(
      shim.localBinarySupportsCommands(binary, ["smoke"], {
        runHelp: () => ({ ok: false, output: "" }),
      }),
      false
    );
  });

  it("requires commands to appear as command entries in help output", () => {
    const temp = mkdtempSync(join(tmpdir(), "architect-tui-help-match-"));
    const binary = join(temp, binaryName);
    writeFileSync(binary, "binary\n");
    chmodSync(binary, 0o755);

    assert.equal(
      shim.localBinarySupportsCommands(binary, ["smoke"], {
        runHelp: () => ({
          ok: true,
          output:
            "Commands:\n  run     Run a secret-safe terminal QA smoke report\n  config  Manage config\n",
        }),
      }),
      false
    );
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

  it("follows https redirects for text and file downloads", async () => {
    const originalGet = https.get;
    let callCount = 0;
    (https as unknown as { get: typeof https.get }).get = ((_url: string, callback: (response: EventEmitter & { statusCode?: number; headers: Record<string, string>; resume: () => void; setEncoding: (value: BufferEncoding) => void; pipe: (destination: NodeJS.WritableStream) => NodeJS.WritableStream }) => void) => {
      callCount += 1;
      const response = new EventEmitter() as EventEmitter & {
        statusCode?: number;
        headers: Record<string, string>;
        resume: () => void;
        setEncoding: (value: BufferEncoding) => void;
        pipe: (destination: NodeJS.WritableStream) => NodeJS.WritableStream;
      };
      response.headers = {};
      response.resume = () => {
        response.emit("end");
      };
      response.setEncoding = () => {};
      let body = "";
      response.pipe = (destination: NodeJS.WritableStream) => {
        if (body) {
          destination.write(body);
        }
        destination.end();
        return destination;
      };
      if (callCount === 1) {
        response.statusCode = 302;
        response.headers.location = "https://example.test/final.sha256";
      } else if (callCount === 2) {
        response.statusCode = 200;
        body = "abc123  file.tgz\n";
      } else if (callCount === 3) {
        response.statusCode = 302;
        response.headers.location = "https://example.test/final.tgz";
      } else {
        response.statusCode = 200;
        body = "archive-content";
      }
      process.nextTick(() => {
        callback(response);
        if (body) {
          response.emit("data", body);
          response.emit("end");
        }
      });
      return new EventEmitter() as unknown as ReturnType<typeof https.get>;
    }) as typeof https.get;

    try {
      const sha = await shim.downloadText("https://example.test/start.sha256");
      assert.equal(sha, "abc123  file.tgz\n");
      const temp = mkdtempSync(join(tmpdir(), "architect-tui-redirect-"));
      const destination = join(temp, "archive.tgz");
      await shim.downloadFile("https://example.test/start.tgz", destination);
      assert.equal(await readFile(destination, "utf8"), "archive-content");
      assert.equal(callCount, 4);
    } finally {
      (https as unknown as { get: typeof https.get }).get = originalGet;
    }
  });
});
