export function isUiFile(path: string): boolean {
  return /\.(tsx|jsx)$/.test(path) || path.includes("/components/") || path.includes("/shared/ui/");
}

export function importsServerOnlyModule(imports: string[]): boolean {
  return imports.some((specifier) =>
    specifier.includes("/server") ||
    specifier.includes("/db") ||
    specifier.includes("server-only") ||
    specifier.includes("service-role")
  );
}

export function dedupeViolations<T extends { severity: string; path?: string; message: string }>(violations: T[]): T[] {
  const seen = new Set<string>();
  return violations.filter((violation) => {
    const key = `${violation.severity}:${violation.path ?? ""}:${violation.message}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
