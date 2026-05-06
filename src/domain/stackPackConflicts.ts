import { inferTriggerKind } from "./stackPacks.js";
import type { StackPack, StackPackConflict } from "./types.js";

export function analyzeStackPackConflicts(packs: StackPack[]): StackPackConflict[] {
  const conflicts: StackPackConflict[] = [];
  const seenRules = new Map<string, { packId: string; ruleName: string }>();
  const seenTriggers = new Map<string, { packId: string; ruleName: string }>();

  for (const pack of packs) {
    for (const rule of pack.fileRules) {
      const normalizedRule = normalizeRule(rule.rule);
      const previousRule = seenRules.get(normalizedRule);
      if (previousRule && previousRule.packId !== pack.id) {
        conflicts.push({
          code: "duplicate-rule",
          severity: "warning",
          packIds: [previousRule.packId, pack.id],
          ruleNames: [previousRule.ruleName, rule.name],
          message: `Rules "${previousRule.ruleName}" and "${rule.name}" describe the same enforcement intent.`,
          resolution: "Keep the stricter stack-specific rule and suppress the duplicate at contract generation time."
        });
      } else {
        seenRules.set(normalizedRule, { packId: pack.id, ruleName: rule.name });
      }

      const triggerKey = `${rule.triggerKind ?? inferTriggerKind(rule.name)}:${(rule.appliesToPaths ?? []).sort().join("|")}`;
      const previousTrigger = seenTriggers.get(triggerKey);
      if (previousTrigger && previousTrigger.packId !== pack.id) {
        conflicts.push({
          code: "overlapping-path-trigger",
          severity: "warning",
          packIds: [previousTrigger.packId, pack.id],
          ruleNames: [previousTrigger.ruleName, rule.name],
          message: `Rules "${previousTrigger.ruleName}" and "${rule.name}" use the same trigger on the same path scope.`,
          resolution: "Prefer a single canonical rule in the contract and keep the other pack's source as supporting evidence."
        });
      } else {
        seenTriggers.set(triggerKey, { packId: pack.id, ruleName: rule.name });
      }
    }
  }

  return conflicts;
}

function normalizeRule(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}
