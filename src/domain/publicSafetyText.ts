export function publicSafeText(value: string): { value: string; redacted: boolean } {
  let redacted = false;
  let safe = value.trim();
  const replacements: Array<[RegExp, string]> = [
    [/\b(?:npm|gh[pousr]|github_pat|sk|xox[baprs])_[A-Za-z0-9_=-]{16,}\b/g, "[redacted-token]"],
    [/\b[A-Za-z0-9_-]{48,}\b/g, "[redacted-token]"],
    [/(?:\/Users|\/home|\/private\/tmp|\/tmp|\/var\/folders|\/Volumes)\/[^\s,;)"']+/g, "[redacted-local-path]"],
    [/\b[A-Za-z]:\\[^\s,;)"']+/g, "[redacted-local-path]"],
    [/\b(?:secret internal path|private stack trace|RAW_PRIVATE_PAYLOAD)\b/gi, "[redacted-sensitive-detail]"]
  ];
  for (const [pattern, replacement] of replacements) {
    safe = safe.replace(pattern, () => {
      redacted = true;
      return replacement;
    });
  }
  return { value: safe, redacted };
}

export function publicSafeOptionalText(value: string | undefined): { value: string | undefined; redacted: boolean } {
  if (value === undefined) return { value: undefined, redacted: false };
  return publicSafeSummary(value);
}

export function publicSafeSummary(value: string): { value: string; redacted: boolean } {
  if (containsRawOutput(value)) {
    return { value: "[redacted-raw-output]", redacted: true };
  }
  return publicSafeText(value);
}

function containsRawOutput(value: string): boolean {
  return /```[\s\S]*?```/.test(value) || /\b(?:stdout|stderr|payload)\s*:/i.test(value);
}
