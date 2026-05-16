import { defineConfig } from "vitepress";

export default defineConfig({
  title: "architect-mcp",
  description: "A local-first agent work gate for coding agents.",
  base: "/architect-mcp/",
  cleanUrls: true,
  ignoreDeadLinks: [/^docs\//, /^examples\//],
  themeConfig: {
    nav: [
      { text: "Guide", link: "/getting-started" },
      { text: "TUI", link: "/rust-tui" },
      { text: "Tools", link: "/tool-reference" },
      { text: "GitHub", link: "https://github.com/tonycdr-prog/architect-mcp" }
    ],
    sidebar: [
      {
        text: "Start",
        items: [
          { text: "Overview", link: "/" },
          { text: "Getting Started", link: "/getting-started" },
          { text: "MCP Client Setup", link: "/mcp-client-setup" },
          { text: "Core Work Gate", link: "/core-work-gate" }
        ]
      },
      {
        text: "TUI Platform",
        items: [
          { text: "Rust TUI", link: "/rust-tui" }
        ]
      },
      {
        text: "Reference",
        items: [
          { text: "Tool Reference", link: "/tool-reference" },
          { text: "Hosted Mode", link: "/hosted-mode" },
          { text: "MCP Integrations", link: "/mcp-integrations" },
          { text: "Stack Packs", link: "/stack-packs" },
          { text: "Release Readiness", link: "/release-readiness" },
          { text: "Read-Only Smoke Matrix", link: "/read-only-smoke-matrix" },
          { text: "Obsidian Project Memory", link: "/obsidian-project-memory" }
        ]
      },
      {
        text: "Compatibility",
        items: [
          { text: "Advanced Maturity Criteria", link: "/compatibility" },
          { text: "Use On A Repo", link: "/use-on-a-repo" },
          { text: "Hosted API Shape", link: "/hosted-api-shape" },
          { text: "Repo Quality Eval", link: "/repo-quality-eval" },
          { text: "Stack Pack Strategy", link: "/stack-pack-strategy" },
          { text: "Architecture Contract", link: "/architecture-contract" },
          { text: "Build Plan", link: "/build-plan" }
        ]
      }
    ],
    socialLinks: [
      { icon: "github", link: "https://github.com/tonycdr-prog/architect-mcp" }
    ],
    search: {
      provider: "local"
    },
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright (c) 2026"
    }
  }
});
