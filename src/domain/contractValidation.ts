import { listStackPacks } from "./stackPacks.js";
import { listFoundationPacks } from "./foundationPacks.js";
import type { ArchitectureContract } from "./types.js";

export type ValidationResult = {
  valid: boolean;
  errors: string[];
  warnings: string[];
};

const REQUIRED_CONTRACT_SECTIONS = [
  "contractVersion",
  "generatedBy",
  "directories",
  "fileRules",
  "moduleBoundaries",
  "testingExpectations",
  "agentInstructions",
  "foundationPacks"
] as const;

export function validateArchitectureContract(contract: ArchitectureContract): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];
  const knownPacks = new Map(listStackPacks().map((pack) => [pack.id, pack]));
  const knownFoundationPacks = new Set(listFoundationPacks().map((pack) => pack.id));

  for (const section of REQUIRED_CONTRACT_SECTIONS) {
    if (!contract[section]) errors.push(`Contract is missing ${section}.`);
  }

  if (!contract.contractVersion?.match(/^\d+\.\d+\.\d+$/)) {
    errors.push("Contract version must be semver.");
  }

  if (contract.generatedBy?.tool !== "architect-mcp") {
    errors.push("Contract generatedBy.tool must be architect-mcp.");
  }

  if (!contract.generatedBy?.version?.match(/^\d+\.\d+\.\d+$/)) {
    errors.push("Contract generatedBy.version must be semver.");
  }

  if (!contract.generatedBy?.generatedAt || Number.isNaN(Date.parse(contract.generatedBy.generatedAt))) {
    errors.push("Contract generatedBy.generatedAt must be an ISO timestamp.");
  }

  for (const pack of contract.stackPacks ?? []) {
    const knownPack = knownPacks.get(pack.id);
    if (!knownPack) {
      errors.push(`Unknown stack pack: ${pack.id}.`);
      continue;
    }

    if (knownPack.version !== pack.version) {
      warnings.push(`Stack pack ${pack.id} version ${pack.version} differs from available version ${knownPack.version}.`);
    }
  }

  for (const pack of contract.foundationPacks ?? []) {
    if (!knownFoundationPacks.has(pack.id)) {
      errors.push(`Unknown foundation pack: ${pack.id}.`);
    }
  }

  for (const requiredPack of knownFoundationPacks) {
    if (!contract.foundationPacks?.some((pack) => pack.id === requiredPack)) {
      errors.push(`Contract is missing foundation pack: ${requiredPack}.`);
    }
  }

  if (!contract.archetype) {
    warnings.push("Contract has no app archetype.");
  }

  if (!contract.directories?.some((directory) => directory.required)) {
    errors.push("Contract must contain at least one required directory.");
  }

  if (!contract.fileRules?.some((rule) => rule.severity === "error")) {
    warnings.push("Contract has no error-level file rules.");
  }

  if (!contract.agentInstructions?.some((instruction) => /review/i.test(instruction))) {
    errors.push("Contract agent instructions must mention architecture review.");
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings
  };
}
