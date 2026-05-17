#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");

const packageJson = require("../package.json");
const MAX_REDIRECTS = 5;
const REQUIRED_LOCAL_HELP_COMMANDS = [
  "smoke",
  "terminal-evidence",
  "walkthrough",
  "promotion-smoke",
  "foundry-smoke",
  "governance-audit",
  "launch-judge",
  "launch-stack",
  "launch-readiness",
  "evidence-index",
  "collect-terminal-evidence",
];

const root = path.resolve(__dirname, "..");
const binaryName = process.platform === "win32" ? "architect-mcp-tui.exe" : "architect-mcp-tui";
const localCandidates = localBinaryCandidates(root);

if (require.main === module) {
  main().catch((error) => {
    console.error(`architect-mcp-tui: ${error.message}`);
    process.exit(1);
  });
}

module.exports = {
  archTag,
  downloadFile,
  downloadText,
  ensureCachedReleaseBinary,
  findLocalBinary,
  isExecutable,
  localBinarySupportsCommands,
  localBinaryCandidates,
  platformTag,
  sha256File,
};

function localBinaryCandidates(baseRoot) {
  return [
    path.join(baseRoot, "target", "release", binaryName),
    path.join(baseRoot, "target", "debug", binaryName),
    path.join(baseRoot, "crates", "architect-tui", "target", "release", binaryName),
    path.join(baseRoot, "crates", "architect-tui", "target", "debug", binaryName),
  ];
}

async function main() {
  const local = findLocalBinary(localCandidates, {
    requiredHelpCommands: REQUIRED_LOCAL_HELP_COMMANDS,
  });
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

function findLocalBinary(candidates, options = {}) {
  const requiredHelpCommands = options.requiredHelpCommands || [];
  return candidates.find(
    (candidate) =>
      isExecutable(candidate) &&
      localBinarySupportsCommands(candidate, requiredHelpCommands, options),
  );
}

function localBinarySupportsCommands(binaryPath, requiredHelpCommands = [], options = {}) {
  if (!requiredHelpCommands.length) {
    return true;
  }
  const help = runBinaryHelp(binaryPath, options);
  if (!help.ok) {
    return false;
  }
  return requiredHelpCommands.every((command) => helpListsCommand(help.output, command));
}

function runBinaryHelp(binaryPath, options = {}) {
  if (options.runHelp) {
    return options.runHelp(binaryPath);
  }
  const result = spawnSync(binaryPath, ["--help"], {
    encoding: "utf8",
    timeout: 5000,
  });
  return {
    ok: !result.error && result.status === 0,
    output: `${result.stdout || ""}\n${result.stderr || ""}`,
  };
}

async function ensureCachedReleaseBinary(options = {}) {
  const cacheRoot =
    options.cacheRoot ||
    process.env.ARCHITECT_MCP_TUI_CACHE_DIR ||
    path.join(os.homedir(), ".cache", "architect-mcp", "tui");
  const platform = options.platform || platformTag();
  const arch = options.arch || archTag();
  const version = options.version || packageJson.version;
  const downloadTextFn = options.downloadText || downloadText;
  const downloadFileFn = options.downloadFile || downloadFile;
  const extractArchiveFn = options.extractArchive || extractArchive;
  const isExecutableFn = options.isExecutable || isExecutable;
  const assetName = `architect-mcp-tui-${platform}-${arch}${process.platform === "win32" ? ".zip" : ".tar.gz"}`;
  const cacheDir = path.join(cacheRoot, version, `${platform}-${arch}`);
  const cachedBinary = path.join(cacheDir, binaryName);
  if (isExecutableFn(cachedBinary)) {
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
    const expectedSha = (await downloadTextFn(shaUrl)).trim().split(/\s+/)[0];
    await downloadFileFn(assetUrl, archivePath);
    const actualSha = sha256File(archivePath);
    if (actualSha !== expectedSha) {
      fs.rmSync(archivePath, { force: true });
      throw new Error(`checksum mismatch for ${assetName}`);
    }
    extractArchiveFn(archivePath, cacheDir);
    if (!isExecutableFn(cachedBinary)) {
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
    getWithRedirects(url, MAX_REDIRECTS, (response, finalUrl) => {
        if (response.statusCode !== 200) {
          reject(new Error(`download failed ${finalUrl}: HTTP ${response.statusCode}`));
          response.resume();
          return;
        }
        response.setEncoding("utf8");
        let body = "";
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => resolve(body));
      }, reject);
  });
}

function downloadFile(url, destination) {
  return new Promise((resolve, reject) => {
    getWithRedirects(url, MAX_REDIRECTS, (response, finalUrl) => {
        if (response.statusCode !== 200) {
          reject(new Error(`download failed ${finalUrl}: HTTP ${response.statusCode}`));
          response.resume();
          return;
        }
        const file = fs.createWriteStream(destination, { mode: 0o755 });
        file.on("error", (error) => {
          fs.rmSync(destination, { force: true });
          reject(error);
        });
        response.pipe(file);
        file.on("finish", () => {
          file.close(resolve);
        });
      }, reject);
  });
}

function getWithRedirects(url, redirectsRemaining, onResponse, reject) {
  https
    .get(url, (response) => {
      if (isRedirectStatus(response.statusCode)) {
        const location = response.headers.location;
        response.resume();
        if (!location) {
          reject(new Error(`download failed ${url}: HTTP ${response.statusCode}`));
          return;
        }
        if (redirectsRemaining <= 0) {
          reject(new Error(`download failed ${url}: too many redirects`));
          return;
        }
        const nextUrl = new URL(location, url);
        if (nextUrl.protocol !== "https:") {
          reject(new Error(`download failed ${url}: redirected to unsupported protocol`));
          return;
        }
        getWithRedirects(nextUrl.toString(), redirectsRemaining - 1, onResponse, reject);
        return;
      }
      onResponse(response, url);
    })
    .on("error", reject);
}

function isRedirectStatus(statusCode) {
  return statusCode === 301 || statusCode === 302 || statusCode === 303 || statusCode === 307 || statusCode === 308;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function helpListsCommand(helpOutput, command) {
  return new RegExp(`^\\s+${escapeRegExp(command)}(?:\\s{2,}.*)?$`, "m").test(helpOutput);
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
