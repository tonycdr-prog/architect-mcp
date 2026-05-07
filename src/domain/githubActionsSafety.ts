const SAFE_COMMANDS = [
  /^npm run [a-z0-9:_-]+$/i,
  /^npm test$/i,
  /^npm audit$/i,
  /^npm ci$/i,
  /^pnpm (install|test|run [a-z0-9:_-]+)$/i,
  /^yarn (install|test|run [a-z0-9:_-]+)$/i,
  /^bun (install|test|run [a-z0-9:_-]+)$/i,
  /^pytest(?:\s+[a-z0-9_./:-]+)*$/i,
  /^python -m pytest(?:\s+[a-z0-9_./:-]+)*$/i,
  /^uv run [a-z0-9_./:-]+(?:\s+[a-z0-9_./:-]+)*$/i,
  /^go test(?:\s+[a-z0-9_./:-]+)*$/i,
  /^cargo test(?:\s+[a-z0-9_./:-]+)*$/i,
  /^dotnet test(?:\s+[a-z0-9_./:-]+)*$/i
];

const UNSAFE_SHELL_TOKENS = /[\n\r;&|`$<>\\]/;

export function validateGithubActionsRunCommand(command: string): { valid: boolean; reason?: string } {
  const normalized = command.trim();
  if (!normalized) return { valid: false, reason: "Verification command is blank." };
  if (normalized !== command) return { valid: false, reason: "Verification command must not include leading or trailing whitespace." };
  if (UNSAFE_SHELL_TOKENS.test(normalized)) return { valid: false, reason: "Verification command must be a single safe command without shell chaining, interpolation, redirection, or newlines." };
  if (!SAFE_COMMANDS.some((pattern) => pattern.test(normalized))) {
    return { valid: false, reason: "Verification command must be an allowed package-manager check." };
  }
  return { valid: true };
}

export function filterSafeGithubActionsRunCommands(commands: string[]): { commands: string[]; rejected: Array<{ command: string; reason: string }> } {
  const safe: string[] = [];
  const rejected: Array<{ command: string; reason: string }> = [];
  for (const command of commands) {
    const result = validateGithubActionsRunCommand(command);
    if (result.valid) {
      safe.push(command);
    } else {
      rejected.push({ command, reason: result.reason ?? "Command was rejected." });
    }
  }
  return {
    commands: [...new Set(safe)],
    rejected
  };
}
