export type JavaScriptPackageManager = "npm" | "pnpm" | "yarn" | "bun";

export const VERIFICATION_COMMAND_PATTERN =
  /^(npm|pnpm|yarn|bun|npx|node|tsx|pytest|python|ruff|uv|cargo|go|dotnet|mvn|gradle|make)\b/i;

export function isConcreteVerificationCommand(command: string): boolean {
  return VERIFICATION_COMMAND_PATTERN.test(command.trim());
}

export function extractVerificationCommands(candidates: string[]): string[] {
  return [...new Set(candidates.map((command) => command.trim()).filter(isConcreteVerificationCommand))];
}

export function packageManagerForVerificationCommands(commands: string[]): JavaScriptPackageManager | undefined {
  if (commands.some((command) => /^pnpm\b/i.test(command))) return "pnpm";
  if (commands.some((command) => /^yarn\b/i.test(command))) return "yarn";
  if (commands.some((command) => /^bun\b/i.test(command))) return "bun";
  if (commands.some((command) => /^(npm|npx|node|tsx)\b/i.test(command))) return "npm";
  return undefined;
}
