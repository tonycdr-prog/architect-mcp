import { publicSafeSummary } from "./publicSafetyText.js";
import type { FoundryEvidenceItem } from "./foundryEvidenceTypes.js";

export type SafeAssessmentIdentity = {
  evidenceId: string;
  sourceType: FoundryEvidenceItem["sourceType"];
  code?: string;
  path?: string;
  redacted: boolean;
};

export function safeAssessmentIdentity(item: FoundryEvidenceItem): SafeAssessmentIdentity {
  const evidenceId = safeRequiredText(item.id, "unknown-evidence");
  const code = safeOptionalText(item.code);
  const path = safeOptionalText(item.path);
  const sourceRefSourceId = safeOptionalText(item.sourceRef?.sourceId);
  const sourceRefPath = safeOptionalText(item.sourceRef?.path);
  const sourceType = isKnownSourceType(item.sourceType) ? item.sourceType : "external_tool";
  const rawSourceType = safeOptionalText(typeof item.sourceType === "string" ? item.sourceType : undefined);
  return {
    evidenceId: evidenceId.value,
    sourceType,
    code: code.value,
    path: path.value,
    redacted: evidenceId.redacted ||
      code.redacted ||
      path.redacted ||
      sourceRefSourceId.redacted ||
      sourceRefPath.redacted ||
      rawSourceType.redacted ||
      !isKnownSourceType(item.sourceType)
  };
}

function safeRequiredText(value: unknown, fallback: string): { value: string; redacted: boolean } {
  const safe = typeof value === "string" ? publicSafeSummary(value) : { value: fallback, redacted: false };
  return { value: safe.value.length > 0 ? safe.value : fallback, redacted: safe.redacted };
}

function safeOptionalText(value: unknown): { value?: string; redacted: boolean } {
  if (typeof value !== "string") return { value: undefined, redacted: false };
  const safe = publicSafeSummary(value);
  return { value: safe.value.length > 0 ? safe.value : undefined, redacted: safe.redacted };
}

function isKnownSourceType(value: unknown): value is FoundryEvidenceItem["sourceType"] {
  return value === "architect_review" ||
    value === "external_tool" ||
    value === "verification" ||
    value === "repo_constitution" ||
    value === "coverage";
}
