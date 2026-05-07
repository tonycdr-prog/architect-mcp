import type { ArchitectureContract } from "./types.js";

export type ContractDiff = {
  breaking: boolean;
  changes: ContractDiffChange[];
};

export type ContractDiffChange = {
  kind:
    | "pack-version"
    | "stack-pack-added"
    | "stack-pack-removed"
    | "required-directory-added"
    | "required-directory-removed"
    | "error-rule-added"
    | "error-rule-removed"
    | "review-gate-guidance";
  severity: "breaking" | "notice";
  message: string;
};

export function diffArchitectureContracts(before: ArchitectureContract, after: ArchitectureContract): ContractDiff {
  const changes: ContractDiffChange[] = [
    ...diffPackVersions(before, after),
    ...diffRequiredDirectories(before, after),
    ...diffErrorRules(before, after),
    ...diffReviewGateGuidance(before, after)
  ];

  return {
    breaking: changes.some((change) => change.severity === "breaking"),
    changes
  };
}

function diffPackVersions(before: ArchitectureContract, after: ArchitectureContract): ContractDiffChange[] {
  const beforePacks = new Map(before.stackPacks.map((pack) => [pack.id, pack.version]));
  const afterPacks = new Map(after.stackPacks.map((pack) => [pack.id, pack.version]));
  const changes: ContractDiffChange[] = [];

  for (const pack of after.stackPacks) {
    const previousVersion = beforePacks.get(pack.id);
    if (!previousVersion) {
      changes.push({
        kind: "stack-pack-added",
        severity: "breaking",
        message: `Stack pack added: ${pack.id}@${pack.version}.`
      });
    } else if (previousVersion !== pack.version) {
      changes.push({
        kind: "pack-version",
        severity: "breaking",
        message: `Stack pack ${pack.id} changed from ${previousVersion} to ${pack.version}.`
      });
    }
  }

  for (const pack of before.stackPacks) {
    if (!afterPacks.has(pack.id)) {
      changes.push({
        kind: "stack-pack-removed",
        severity: "breaking",
        message: `Stack pack removed: ${pack.id}@${pack.version}.`
      });
    }
  }

  return changes;
}

function diffRequiredDirectories(before: ArchitectureContract, after: ArchitectureContract): ContractDiffChange[] {
  const beforeDirs = new Set(before.directories.filter((directory) => directory.required).map((directory) => directory.path));
  const afterDirs = new Set(after.directories.filter((directory) => directory.required).map((directory) => directory.path));

  return [
    ...[...afterDirs].filter((path) => !beforeDirs.has(path)).map<ContractDiffChange>((path) => ({
      kind: "required-directory-added",
      severity: "breaking",
      message: `Required directory added: ${path}.`
    })),
    ...[...beforeDirs].filter((path) => !afterDirs.has(path)).map<ContractDiffChange>((path) => ({
      kind: "required-directory-removed",
      severity: "notice",
      message: `Required directory removed: ${path}.`
    }))
  ];
}

function diffErrorRules(before: ArchitectureContract, after: ArchitectureContract): ContractDiffChange[] {
  const beforeRules = new Set(before.fileRules.filter((rule) => rule.severity === "error").map((rule) => rule.name));
  const afterRules = new Set(after.fileRules.filter((rule) => rule.severity === "error").map((rule) => rule.name));

  return [
    ...[...afterRules].filter((name) => !beforeRules.has(name)).map<ContractDiffChange>((name) => ({
      kind: "error-rule-added",
      severity: "breaking",
      message: `Error-level rule added: ${name}.`
    })),
    ...[...beforeRules].filter((name) => !afterRules.has(name)).map<ContractDiffChange>((name) => ({
      kind: "error-rule-removed",
      severity: "notice",
      message: `Error-level rule removed: ${name}.`
    }))
  ];
}

function diffReviewGateGuidance(before: ArchitectureContract, after: ArchitectureContract): ContractDiffChange[] {
  const beforeGuidance = before.agentInstructions.filter((instruction) => /gate|review|baseline/i.test(instruction)).join("\n");
  const afterGuidance = after.agentInstructions.filter((instruction) => /gate|review|baseline/i.test(instruction)).join("\n");
  if (beforeGuidance === afterGuidance) return [];

  return [
    {
      kind: "review-gate-guidance",
      severity: "notice",
      message: "Review gate or baseline guidance changed."
    }
  ];
}
