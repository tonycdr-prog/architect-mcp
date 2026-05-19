use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use uuid::Uuid;

pub(crate) fn create_smoke_workspace() -> Result<PathBuf> {
    let workspace =
        std::env::temp_dir().join(format!("architect-mcp-tui-promotion-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)
        .with_context(|| format!("failed to create {}", workspace.display()))?;
    Ok(workspace)
}

pub(crate) fn init_git_workspace(workspace: &Path) -> Result<()> {
    run_git(workspace, ["init"])?;
    run_git(
        workspace,
        ["config", "user.email", "promotion-smoke@example.com"],
    )?;
    run_git(
        workspace,
        ["config", "user.name", "architect-mcp promotion smoke"],
    )?;
    fs::write(
        workspace.join("README.md"),
        "architect-mcp promotion smoke\n",
    )?;
    fs::write(
        workspace.join("package.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "private": true,
            "scripts": {
                "test": "node -e \"const fs=require('node:fs'); const p='docs/codex-adapter-smoke.md'; if(!fs.existsSync(p)){console.error('missing '+p); process.exit(1)} const text=fs.readFileSync(p,'utf8'); if(!text.includes('codex-adapter-smoke')){console.error('missing codex-adapter-smoke marker'); process.exit(1)}\""
            }
        }))?,
    )?;
    run_git(workspace, ["add", "README.md", "package.json"])?;
    run_git(workspace, ["commit", "-m", "init"])?;
    Ok(())
}

fn run_git<const N: usize>(workspace: &Path, args: [&str; N]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
