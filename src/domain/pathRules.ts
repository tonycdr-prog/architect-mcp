export const GENERATED_FILE_PATTERNS = [
  /^package-lock\.json$/,
  /^pnpm-lock\.yaml$/,
  /^yarn\.lock$/,
  /^dist\//,
  /^build\//,
  /^coverage\//,
  /^node_modules\//,
  /^stack-sources\/ingested\//,
  /^ios\/Pods\//,
  /^android\/build\//,
  /^test-output\//,
  /^docs-site\/out\//,
  /^.*\.snap$/
];

const BINARY_OR_MEDIA_EXTENSIONS = /\.(png|jpe?g|gif|webp|avif|ico|mov|mp4|webm|mp3|wav|pdf|zip|gz|tgz|sqlite|db)$/i;

export function normalizePath(path: string): string {
  return path.replaceAll("\\", "/").replace(/^\.?\//, "");
}

export function isGeneratedFile(path: string): boolean {
  const normalizedPath = normalizePath(path);
  return BINARY_OR_MEDIA_EXTENSIONS.test(normalizedPath) || GENERATED_FILE_PATTERNS.some((pattern) => pattern.test(normalizedPath));
}

export function isSourceCodeFile(path: string): boolean {
  return /\.(ts|tsx|js|jsx|mts|cts|mjs|cjs)$/.test(normalizePath(path));
}

export function matchesPathPattern(path: string, pattern: string): boolean {
  const normalizedPath = normalizePath(path);
  const normalizedPattern = normalizePath(pattern);
  const regex = new RegExp(`^${globToRegex(normalizedPattern)}$`);
  return regex.test(normalizedPath);
}

export function hasGlobSyntax(pattern: string): boolean {
  try {
    new RegExp(`^${globToRegex(normalizePath(pattern))}$`);
    return true;
  } catch {
    return false;
  }
}

function globToRegex(pattern: string): string {
  let output = "";

  for (let index = 0; index < pattern.length; index += 1) {
    const char = pattern[index];
    const nextChar = pattern[index + 1];
    const previousChar = pattern[index - 1];
    const afterGlobstar = pattern[index + 2];

    if (char === "*" && nextChar === "*") {
      if (previousChar === "/" && afterGlobstar === "/") {
        output = output.slice(0, -1);
        output += "(?:.*/)?";
        index += 2;
        continue;
      }
      output += ".*";
      index += 1;
      continue;
    }

    if (char === "*") {
      output += "[^/]*";
      continue;
    }

    output += char.replace(/[|\\{}()[\]^$+?.]/g, "\\$&");
  }

  return output;
}
