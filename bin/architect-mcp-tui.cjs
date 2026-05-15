#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");

const packageJson = require("../package.json");

const root = path.resolve(__dirname, "..");
const binaryName = process.platform === "win32" ? "architect-mcp-tui.exe" : "architect-mcp-tui";
const localCandidates = [
  path.join(root, "target", "release", binaryName),
  path.join(root, "target", "debug", binaryName),
  path.join(root, "crates", "architect-tui", "target", "release", binaryName),
  path.join(root, "crates", "architect-tui", "target", "debug", binaryName),
];

main().catch((error) => {
  console.error(`architect-mcp-tui: ${error.message}`);
  process.exit(1);
});

async function main() {
  const local = localCandidates.find(isExecutable);
  if (local) {
    return execBinary(local);
  }

  const cached = await ensureCachedReleaseBinary();
  return execBinary(cached);
}

function execBinary(binaryPath) {
  const child = spawn(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    }
    process.exit(code ?? 1);
  });
  child.on("error", (error) => {
    console.error(`architect-mcp-tui: failed to start ${binaryPath}: ${error.message}`);
    process.exit(1);
  });
}

async function ensureCachedReleaseBinary() {
  const cacheRoot =
    process.env.ARCHITECT_MCP_TUI_CACHE_DIR ||
    path.join(os.homedir(), ".cache", "architect-mcp", "tui");
  const platform = platformTag();
  const arch = archTag();
  const version = packageJson.version;
  const assetName = `architect-mcp-tui-${platform}-${arch}${process.platform === "win32" ? ".zip" : ".tar.gz"}`;
  const cacheDir = path.join(cacheRoot, version, `${platform}-${arch}`);
  const cachedBinary = path.join(cacheDir, binaryName);
  if (isExecutable(cachedBinary)) {
    return cachedBinary;
  }

  fs.mkdirSync(cacheDir, { recursive: true });
  const baseUrl =
    process.env.ARCHITECT_MCP_TUI_RELEASE_BASE ||
    `https://github.com/tonycdr-prog/architect-mcp/releases/download/v${version}`;
  const assetUrl = `${baseUrl}/${assetName}`;
  const shaUrl = `${assetUrl}.sha256`;
  const archivePath = path.join(cacheDir, assetName);

  try {
    const expectedSha = (await downloadText(shaUrl)).trim().split(/\s+/)[0];
    await downloadFile(assetUrl, archivePath);
    const actualSha = sha256File(archivePath);
    if (actualSha !== expectedSha) {
      fs.rmSync(archivePath, { force: true });
      throw new Error(`checksum mismatch for ${assetName}`);
    }
    extractArchive(archivePath, cacheDir);
    if (!isExecutable(cachedBinary)) {
      throw new Error(`release archive did not contain ${binaryName}`);
    }
    return cachedBinary;
  } catch (error) {
    throw new Error(
      `${error.message}. Build locally with "npm run tui:build" or set ARCHITECT_MCP_TUI_RELEASE_BASE.`,
    );
  }
}

function platformTag() {
  if (process.platform === "darwin") return "macos";
  if (process.platform === "linux") return "linux";
  if (process.platform === "win32") return "windows";
  throw new Error(`unsupported platform ${process.platform}`);
}

function archTag() {
  if (process.arch === "x64") return "x64";
  if (process.arch === "arm64") return "arm64";
  throw new Error(`unsupported architecture ${process.arch}`);
}

function isExecutable(candidate) {
  try {
    fs.accessSync(candidate, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function sha256File(filePath) {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

function downloadText(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (response.statusCode !== 200) {
          reject(new Error(`download failed ${url}: HTTP ${response.statusCode}`));
          response.resume();
          return;
        }
        response.setEncoding("utf8");
        let body = "";
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => resolve(body));
      })
      .on("error", reject);
  });
}

function downloadFile(url, destination) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destination, { mode: 0o755 });
    https
      .get(url, (response) => {
        if (response.statusCode !== 200) {
          reject(new Error(`download failed ${url}: HTTP ${response.statusCode}`));
          response.resume();
          return;
        }
        response.pipe(file);
        file.on("finish", () => {
          file.close(resolve);
        });
      })
      .on("error", reject);
  });
}

function extractArchive(archivePath, destination) {
  if (archivePath.endsWith(".zip")) {
    const result = spawnSync("powershell", [
      "-NoProfile",
      "-Command",
      `Expand-Archive -LiteralPath ${JSON.stringify(archivePath)} -DestinationPath ${JSON.stringify(destination)} -Force`,
    ], { stdio: "inherit" });
    assertSpawnOk(result, "powershell Expand-Archive");
    return;
  }

  const result = spawnSync("tar", ["-xzf", archivePath, "-C", destination], { stdio: "inherit" });
  assertSpawnOk(result, "tar");
  fs.chmodSync(path.join(destination, binaryName), 0o755);
}

function assertSpawnOk(result, label) {
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`${label} exited with ${result.status}`);
  }
}
