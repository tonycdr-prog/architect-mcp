import type { RepoLayout, StackProfile } from "./types.js";

export type RepoProfile = {
  id: string;
  name: string;
  description: string;
  stack: StackProfile;
  stackPackIds: string[];
  repoLayout: RepoLayout;
  ignorePatterns: string[];
};

const PROFILES: RepoProfile[] = [
  {
    id: "expo-react-native",
    name: "Expo React Native",
    description: "Expo/React Native app with client-owned screens, components, hooks, and local data modules.",
    stack: { frontend: "React Native Expo" },
    stackPackIds: ["react"],
    repoLayout: {
      pathMap: {
        "src/features": ["client"],
        "src/shared/ui": ["client/components"],
        "src/shared": ["shared"]
      }
    },
    ignorePatterns: ["client/db/types.ts", "**/__tests__/**"]
  },
  {
    id: "express-drizzle",
    name: "Express + Drizzle",
    description: "Node/Express API with Drizzle/Postgres schema and server-side workflows.",
    stack: { backend: "Node API Express TypeScript", database: "Postgres Drizzle" },
    stackPackIds: ["node-api", "postgres"],
    repoLayout: {
      pathMap: {
        "src/server": ["server"],
        "src/server/routes": ["server/routes"],
        "src/server/services": ["server/services", "server/lib"],
        "src/server/adapters": ["server/lib", "server/services"],
        "src/db": ["shared"],
        "src/db/schema": ["shared"],
        "src/db/migrations": ["server/migrations"],
        "src/db/repositories": ["server/services", "server/lib"]
      }
    },
    ignorePatterns: ["server/routes/__tests__/**"]
  },
  {
    id: "supabase",
    name: "Supabase",
    description: "Supabase-backed database/auth project.",
    stack: { database: "Supabase Postgres", auth: "Supabase Auth" },
    stackPackIds: ["postgres", "supabase"],
    repoLayout: {
      pathMap: {
        "src/db": ["supabase", "shared"],
        "src/db/schema": ["shared"],
        "src/db/migrations": ["server/migrations"],
        "src/db/supabase": ["supabase"]
      }
    },
    ignorePatterns: []
  },
  {
    id: "mixed-monorepo",
    name: "Mixed Monorepo",
    description: "Existing production repo with multiple top-level app, server, shared, docs, and script surfaces.",
    stack: { frontend: "React", backend: "Node API", database: "Postgres" },
    stackPackIds: ["react", "node-api", "postgres"],
    repoLayout: {
      pathMap: {
        "src/app": ["admin/react"],
        "src/features": ["client", "admin/react/pages"],
        "src/shared": ["shared"],
        "src/shared/ui": ["client/components", "admin/react/components"],
        "src/server": ["server"],
        "src/server/routes": ["server/routes"],
        "src/server/services": ["server/services", "server/lib"],
        "src/server/adapters": ["server/lib", "server/services"],
        "src/db": ["shared", "supabase"],
        "src/db/schema": ["shared"],
        "src/db/migrations": ["server/migrations"],
        "src/db/repositories": ["server/services", "server/lib"],
        "src/db/supabase": ["supabase"],
        "tests": ["client", "server", "admin"]
      }
    },
    ignorePatterns: ["docs/audits/**/*.json", "**/__tests__/**", "__mocks__/**", "config/**/*.json"]
  },
  {
    id: "static-web-surfaces",
    name: "Static Web Surfaces",
    description: "Static JS/HTML widgets and marketing/report pages.",
    stack: { frontend: "Static web JavaScript" },
    stackPackIds: ["react"],
    repoLayout: {
      pathMap: {
        "src/features": ["chat", "embed", "landing", "pricing", "report", "sift", "site-docs"],
        "src/shared/ui": ["site-docs", "chat", "embed"]
      }
    },
    ignorePatterns: ["**/*.min.js"]
  }
];

export function listRepoProfiles(): RepoProfile[] {
  return PROFILES;
}

export function getRepoProfile(profileId: string): RepoProfile | undefined {
  return PROFILES.find((profile) => profile.id === profileId);
}
