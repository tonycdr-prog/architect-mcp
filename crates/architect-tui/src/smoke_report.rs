use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::adapter::adapter_healths;
use crate::adapter_probe_command::output_with_timeout;
use crate::config::TuiConfig;
use crate::smoke_types::{
    AdapterSummary, BinarySummary, CommandCheck, EnvironmentSummary, SmokeReport,
};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) fn help_check() -> CommandCheck {
    match env::current_exe() {
        Ok(exe) => {
            let command = format!("{} --help", exe.display());
            let mut process = Command::new(exe);
            process.arg("--help");
            run_command(command, &mut process)
        }
        Err(error) => CommandCheck {
            command: "current executable --help".to_string(),
            ok: false,
            exit_code: None,
            timed_out: false,
            first_line: Some(format!("could not resolve current executable: {error}")),
        },
    }
}

pub(crate) fn adapter_summary(config: &TuiConfig) -> AdapterSummary {
    let health = adapter_healths(&config.adapters);
    AdapterSummary {
        total: health.len(),
        ready: health.iter().filter(|health| health.ready).count(),
        health,
    }
}

pub(crate) fn environment_summary(workspace: &Path) -> EnvironmentSummary {
    EnvironmentSummary {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        family: env::consts::FAMILY.to_string(),
        current_dir: env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| workspace.display().to_string()),
        terminal: terminal_summary(),
        tools: tool_versions(),
    }
}

pub(crate) fn binary_summary() -> BinarySummary {
    let path = env::current_exe().ok();
    let npm_cache_env_override = env::var("ARCHITECT_MCP_TUI_CACHE_DIR").ok();
    let npm_cache_root = npm_cache_env_override
        .clone()
        .map(PathBuf::from)
        .or_else(default_npm_cache_root);
    let source = binary_source(path.as_deref(), npm_cache_root.as_deref());
    BinarySummary {
        sha256: path.as_deref().and_then(file_sha256),
        path: path.as_ref().map(|path| path.display().to_string()),
        source,
        npm_cache_root: npm_cache_root.map(|path| path.display().to_string()),
        npm_cache_env_override,
    }
}

pub(crate) fn tail_lines(lines: &[String], max_lines: usize) -> Vec<String> {
    lines
        .iter()
        .skip(lines.len().saturating_sub(max_lines))
        .map(|line| truncate_for_report(line, 800))
        .collect()
}

pub(crate) fn print_human_report(report: &SmokeReport) {
    println!("architect-mcp-tui smoke: {:?}", report.status);
    println!("workspace: {}", report.workspace);
    if let Some(path) = &report.binary.path {
        println!("binary: {path}");
    }
    if let Some(sha) = &report.binary.sha256 {
        println!("binary sha256: {sha}");
    }
    if let Some(cache) = &report.binary.npm_cache_root {
        println!("npm cache root: {cache}");
    }
    println!("help: {}", pass_fail(report.help.ok));
    println!(
        "adapters: {}/{} ready",
        report.adapters.ready, report.adapters.total
    );
    for health in &report.adapters.health {
        println!(
            "- {}: ready={} installed={} auth={:?} detail={}",
            health.name, health.ready, health.installed, health.auth_status, health.detail
        );
    }
    println!(
        "gate-only JSONL: {} status={}",
        pass_fail(report.gate_only.ok),
        report.gate_only.status.as_deref().unwrap_or("unknown")
    );
    if let Some(error) = &report.gate_only.error {
        println!("gate error: {error}");
    }
}

fn terminal_summary() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for key in ["TERM", "COLORTERM", "TERM_PROGRAM", "WT_SESSION", "CI"] {
        if let Ok(value) = env::var(key) {
            values.insert(key.to_ascii_lowercase(), value);
        }
    }
    if let Ok(shell) = env::var("SHELL") {
        values.insert("shell".to_string(), file_name_or_value(&shell));
    }
    if let Ok(comspec) = env::var("ComSpec") {
        values.insert("comspec".to_string(), file_name_or_value(&comspec));
    }
    values
}

fn tool_versions() -> BTreeMap<String, CommandCheck> {
    [
        ("node", ["--version"].as_slice()),
        ("npm", ["--version"].as_slice()),
        ("rustc", ["--version"].as_slice()),
        ("cargo", ["--version"].as_slice()),
    ]
    .into_iter()
    .map(|(command, args)| {
        let mut process = Command::new(command);
        process.args(args);
        (
            command.to_string(),
            run_command(format!("{command} {}", args.join(" ")), &mut process),
        )
    })
    .collect()
}

fn file_name_or_value(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .to_string()
}

fn binary_source(path: Option<&Path>, npm_cache_root: Option<&Path>) -> String {
    let Some(path) = path else {
        return "unknown".to_string();
    };
    if npm_cache_root.is_some_and(|cache| path.starts_with(cache)) {
        return "npm_release_cache".to_string();
    }
    if path
        .components()
        .any(|component| component.as_os_str().to_str() == Some("target"))
    {
        return "source_build".to_string();
    }
    "direct_binary".to_string()
}

fn default_npm_cache_root() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|home| home.join(".cache").join("architect-mcp").join("tui"))
}

fn file_sha256(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Some(hex_bytes(&hasher.finalize()))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn run_command(command: String, process: &mut Command) -> CommandCheck {
    match output_with_timeout(process, COMMAND_TIMEOUT) {
        Ok(Some(output)) => {
            let text = joined_output(&output.stdout, &output.stderr);
            CommandCheck {
                command,
                ok: output.status.success(),
                exit_code: output.status.code(),
                timed_out: false,
                first_line: first_line(&text),
            }
        }
        Ok(None) => CommandCheck {
            command,
            ok: false,
            exit_code: None,
            timed_out: true,
            first_line: Some("command timed out".to_string()),
        },
        Err(error) => CommandCheck {
            command,
            ok: false,
            exit_code: None,
            timed_out: false,
            first_line: Some(error.to_string()),
        },
    }
}

fn joined_output(stdout: &[u8], stderr: &[u8]) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    )
}

fn first_line(text: &str) -> Option<String> {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
}

fn truncate_for_report(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("[truncated]");
    truncated
}

fn pass_fail(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_summary_uses_shell_file_name_only() {
        assert_eq!(file_name_or_value("/bin/zsh"), "zsh");
        assert_eq!(file_name_or_value("pwsh"), "pwsh");
    }

    #[test]
    fn hex_bytes_uses_lowercase_sha_format() {
        assert_eq!(hex_bytes(&[0, 15, 16, 255]), "000f10ff");
    }

    #[test]
    fn report_tail_truncates_large_jsonl_lines() {
        let lines = vec!["x".repeat(900)];
        assert_eq!(tail_lines(&lines, 1)[0].chars().count(), 811);
    }
}
