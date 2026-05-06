export function extractEvidenceLines(text: string): string[] {
  return text.split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => /^#{1,3}\s/.test(line) || /^-\s+\[[^\]]+\]/.test(line))
    .filter((line) => /architecture|routing|route|server|client|component|database|schema|migration|auth|session|deploy|testing|validation|middleware|security|api|docs|guide/i.test(line))
    .slice(0, 20);
}
