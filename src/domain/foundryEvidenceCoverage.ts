import { publicSafeSummary, publicSafeText } from "./publicSafetyText.js";
import type { FoundryEvidenceInventory } from "./foundryEvidenceTypes.js";
import type { ReviewCoverage, ReviewReport } from "./types.js";

export function summarizeFoundryEvidenceCoverage(reports: ReviewReport[]): FoundryEvidenceInventory["coverage"] {
  const coverages = reports.map((report) => report.coverage).filter(Boolean) as ReviewCoverage[];
  return {
    scanTruncated: coverages.some((coverage) => coverage.scanTruncated),
    detailedFindingsTruncated: coverages.some((coverage) => coverage.detailedFindingsTruncated),
    filesReviewed: sumDefined(coverages.map((coverage) => coverage.filesReviewed)),
    maxFiles: maxDefined(coverages.map((coverage) => coverage.maxFiles)),
    topScannedDirectories: mergeDirectories(coverages.flatMap((coverage) => coverage.topScannedDirectories ?? [])),
    findingHistogram: mergeHistograms(coverages.flatMap((coverage) => coverage.findingHistogram ?? [])),
    caveats: [...new Set(coverages.flatMap((coverage) => coverage.caveats ?? []).map((caveat) => publicSafeSummary(caveat).value))]
  };
}

function mergeDirectories(entries: Array<{ directory: string; files: number }>): Array<{ directory: string; files: number }> {
  const counts = new Map<string, number>();
  for (const entry of entries) {
    const directory = publicSafeText(entry.directory).value;
    counts.set(directory, (counts.get(directory) ?? 0) + entry.files);
  }
  return [...counts.entries()].map(([directory, files]) => ({ directory, files })).sort((a, b) => b.files - a.files || a.directory.localeCompare(b.directory)).slice(0, 12);
}

function mergeHistograms(entries: Array<{ code: string; severity: string; count: number }>): Array<{ code: string; severity: string; count: number }> {
  const counts = new Map<string, { code: string; severity: string; count: number }>();
  for (const entry of entries) {
    const code = publicSafeText(entry.code).value;
    const severity = publicSafeText(entry.severity).value;
    const key = `${code}:${severity}`;
    const current = counts.get(key) ?? { code, severity, count: 0 };
    current.count += entry.count;
    counts.set(key, current);
  }
  return [...counts.values()].sort((a, b) => b.count - a.count || a.code.localeCompare(b.code) || a.severity.localeCompare(b.severity));
}

function sumDefined(values: Array<number | undefined>): number | undefined {
  const present = values.filter((value): value is number => value !== undefined);
  return present.length ? present.reduce((sum, value) => sum + value, 0) : undefined;
}

function maxDefined(values: Array<number | undefined>): number | undefined {
  const present = values.filter((value): value is number => value !== undefined);
  return present.length ? Math.max(...present) : undefined;
}
