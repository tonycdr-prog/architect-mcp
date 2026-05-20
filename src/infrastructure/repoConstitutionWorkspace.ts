import { realpathSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { isAbsolute, join, relative } from "node:path";
import {
  deriveRepoConstitution,
  isRepoConstitutionArtifactPath,
  type RepoConstitutionArtifact,
  type RepoConstitutionPullRequest
} from "../domain/repoConstitution.js";
import { scanWorkspaceWithMetadata } from "./scanWorkspace.js";

export async function deriveLocalRepoConstitution(input: {
  rootPath: string;
  maxFiles: number;
  allowOutsideCwd?: boolean;
  recentPullRequests?: RepoConstitutionPullRequest[];
}) {
  assertLocalScanAllowed(input.rootPath, { allowOutsideCwd: input.allowOutsideCwd });
  const scan = await scanWorkspaceWithMetadata(input.rootPath, input.maxFiles);
  const artifacts = await readConstitutionArtifacts(input.rootPath, scan.files.map((file) => file.path));
  return {
    filesReviewed: scan.files.length,
    scan: {
      maxFiles: scan.maxFiles,
      truncated: scan.truncated
    },
    constitution: deriveRepoConstitution({
      files: scan.files,
      artifacts,
      recentPullRequests: input.recentPullRequests
    })
  };
}

async function readConstitutionArtifacts(rootPath: string, filePaths: string[]): Promise<RepoConstitutionArtifact[]> {
  const artifacts: RepoConstitutionArtifact[] = [];
  for (const path of filePaths.filter(isRepoConstitutionArtifactPath).sort()) {
    try {
      const content = await readFile(join(rootPath, path), "utf8");
      artifacts.push({
        path,
        content: content.slice(0, 100_000)
      });
    } catch {
      // The scan can race with local edits; missing or unreadable optional artifacts are omitted.
    }
  }
  return artifacts;
}

function assertLocalScanAllowed(rootPath: string, options: { allowOutsideCwd?: boolean }): void {
  const realRoot = realpathSync(rootPath);
  const realCwd = realpathSync(process.cwd());
  if (isInsideOrEqual(realRoot, realCwd)) return;
  if (options.allowOutsideCwd) return;
  throw new Error(`Refusing to scan ${realRoot}; derive_local_repo_constitution only scans inside ${realCwd} unless allowOutsideCwd is explicitly set for a trusted local run.`);
}

function isInsideOrEqual(child: string, parent: string): boolean {
  const rel = relative(parent, child);
  return rel === "" || (!rel.startsWith("..") && !isAbsolute(rel) && rel !== "..");
}
