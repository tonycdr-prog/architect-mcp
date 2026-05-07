#!/usr/bin/env node
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const ignoredDirs = new Set([".git", "node_modules", "dist", "coverage"]);
const ignoredFiles = new Set(["package-lock.json"]);
const allowedFixturePaths = [/^tests\//, /^docs\//, /^stack-sources\//];
const secretPatterns = [
  /sk-[A-Za-z0-9]{20,}/,
  /(ghp_|gho_|ghu_|ghs_|ghr_)[A-Za-z0-9]{30,}/,
  /AKIA[0-9A-Z]{16}/,
  /-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----/i,
  /[a-z]+:\/\/[^:\s]+:[^@\s]{6,}@/i,
  /(api[_-]?key|token|secret|password|credential)\s*[:=]\s*["']?(?!\$\{|example|placeholder|changeme|replace_me)[^"'\s]{12,}/i
];

const findings = [];
for (const file of walk(".")) {
  const normalized = file.replace(/^\.\//, "");
  if (allowedFixturePaths.some((pattern) => pattern.test(normalized))) continue;
  const content = readFileSync(file, "utf8");
  for (const pattern of secretPatterns) {
    for (const match of content.matchAll(new RegExp(pattern.source, pattern.flags.includes("g") ? pattern.flags : `${pattern.flags}g`))) {
      if (/process\.env|import\.meta\.env|\$\{[A-Z0-9_]+\}/i.test(match[0])) continue;
      findings.push(`${normalized}: matches ${pattern.source}`);
      break;
    }
  }
}

if (findings.length) {
  console.error("Secret scan failed:");
  for (const finding of findings) console.error(`- ${finding}`);
  process.exit(1);
}

console.log("Secret scan passed.");

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    if (ignoredDirs.has(entry) || ignoredFiles.has(entry)) continue;
    const path = join(dir, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      yield* walk(path);
      continue;
    }
    if (stats.isFile() && /\.(ts|tsx|js|mjs|json|md|yml|yaml|env|example)$/.test(path)) {
      yield path;
    }
  }
}
