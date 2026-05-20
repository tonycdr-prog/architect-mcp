import { publicSafeSummary, publicSafeText } from "./publicSafetyText.js";
import type { FoundryEvidenceInventory } from "./foundryEvidenceTypes.js";
import type { ReviewCoverage, ReviewReport } from "./types.js";

export function summarizeFoundryEvidenceCoverage(reports: ReviewReport[]): FoundryEvidenceInventory["coverage"] {
  const coverages = reports.map((report) => report.coverage).filter(Boolean) as ReviewCoverage[];
  const mergedDirectories = mergeDirectories(coverages.flatMap((coverage) => coverage.topScannedDirectories ?? []));
  const mergedHistograms = mergeHistograms(coverages.flatMap((coverage) => coverage.findingHistogram ?? []));
  const mergedCaveats = mergeCaveats(coverages.flatMap((coverage) => coverage.caveats ?? []));
  const caveats = [...mergedCaveats.values];
  if (mergedDirectories.redactedBucketsMerged > 0) {
    caveats.push("Some redacted top-scanned directory entries were merged to preserve public safety.");
  }
  if (mergedHistograms.redactedBucketsMerged > 0) {
    caveats.push("Some redacted finding-histogram buckets were merged to preserve public safety.");
  }
  if (mergedCaveats.redactedEntriesMerged > 0) {
    caveats.push("Some redacted coverage caveats were merged to preserve public safety.");
  }
  return {
    scanTruncated: coverages.some((coverage) => coverage.scanTruncated),
    detailedFindingsTruncated: coverages.some((coverage) => coverage.detailedFindingsTruncated),
    filesReviewed: sumDefined(coverages.map((coverage) => coverage.filesReviewed)),
    maxFiles: maxDefined(coverages.map((coverage) => coverage.maxFiles)),
    topScannedDirectories: mergedDirectories.entries,
    findingHistogram: mergedHistograms.entries,
    caveats
  };
}

function mergeDirectories(entries: Array<{ directory: string; files: number }>): {
  entries: Array<{ directory: string; files: number }>;
  redactedBucketsMerged: number;
} {
  const counts = new Map<string, number>();
  const redactedRawValues = new Map<string, Set<string>>();
  for (const entry of entries) {
    const safeDirectory = publicSafeText(entry.directory);
    const directory = safeDirectory.value;
    counts.set(directory, (counts.get(directory) ?? 0) + entry.files);
    if (safeDirectory.redacted) {
      const seen = redactedRawValues.get(directory) ?? new Set<string>();
      seen.add(entry.directory);
      redactedRawValues.set(directory, seen);
    }
  }
  return {
    entries: [...counts.entries()].map(([directory, files]) => ({ directory, files })).sort((a, b) => b.files - a.files || a.directory.localeCompare(b.directory)).slice(0, 12),
    redactedBucketsMerged: [...redactedRawValues.values()].reduce((sum, values) => sum + Math.max(0, values.size - 1), 0)
  };
}

function mergeHistograms(entries: Array<{ code: string; severity: string; count: number }>): {
  entries: Array<{ code: string; severity: string; count: number }>;
  redactedBucketsMerged: number;
} {
  const counts = new Map<string, { code: string; severity: string; count: number }>();
  const redactedRawValues = new Map<string, Set<string>>();
  for (const entry of entries) {
    const safeCode = publicSafeText(entry.code);
    const safeSeverity = publicSafeText(entry.severity);
    const code = safeCode.value;
    const severity = safeSeverity.value;
    const key = `${code}:${severity}`;
    const current = counts.get(key) ?? { code, severity, count: 0 };
    current.count += entry.count;
    counts.set(key, current);
    if (safeCode.redacted || safeSeverity.redacted) {
      const seen = redactedRawValues.get(key) ?? new Set<string>();
      seen.add(`${entry.code}:${entry.severity}`);
      redactedRawValues.set(key, seen);
    }
  }
  return {
    entries: [...counts.values()].sort((a, b) => b.count - a.count || a.code.localeCompare(b.code) || a.severity.localeCompare(b.severity)),
    redactedBucketsMerged: [...redactedRawValues.values()].reduce((sum, values) => sum + Math.max(0, values.size - 1), 0)
  };
}

function mergeCaveats(entries: string[]): { values: string[]; redactedEntriesMerged: number } {
  const rawBySafe = new Map<string, Set<string>>();
  for (const entry of entries) {
    const safe = publicSafeSummary(entry);
    const seen = rawBySafe.get(safe.value) ?? new Set<string>();
    seen.add(entry);
    rawBySafe.set(safe.value, seen);
  }
  return {
    values: [...rawBySafe.keys()],
    redactedEntriesMerged: [...rawBySafe.entries()].reduce((sum, [safe, values]) => (
      safe.includes("[redacted") ? sum + Math.max(0, values.size - 1) : sum
    ), 0)
  };
}

function sumDefined(values: Array<number | undefined>): number | undefined {
  const present = values.filter((value): value is number => value !== undefined);
  return present.length ? present.reduce((sum, value) => sum + value, 0) : undefined;
}

function maxDefined(values: Array<number | undefined>): number | undefined {
  const present = values.filter((value): value is number => value !== undefined);
  return present.length ? Math.max(...present) : undefined;
}
