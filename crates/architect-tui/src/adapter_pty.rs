use std::io::Read;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

use crate::adapter::{AdapterConfig, AgentEvent};

const OUTPUT_LIMIT_BYTES: usize = 512 * 1024;
const OUTPUT_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub struct PtyRunOptions {
    pub adapter_name: String,
    pub prompt: String,
    pub timeout: Duration,
}

pub fn run_adapter_pty(config: &AdapterConfig, options: PtyRunOptions) -> Result<Vec<AgentEvent>> {
    let mut events = vec![AgentEvent::Started {
        adapter: options.adapter_name,
    }];
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("failed to open PTY")?;

    let mut command = CommandBuilder::new(&config.command);
    for arg in &config.args {
        command.arg(arg);
    }
    command.arg(options.prompt);
    for (key, value) in &config.env {
        command.env(key, value);
    }
    if let Some(cwd) = &config.working_directory {
        command.cwd(cwd);
    }

    let mut child = pair
        .slave
        .spawn_command(command)
        .context("failed to spawn adapter in PTY")?;
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader()?;
    let (output_tx, output_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut output = String::new();
        let mut truncated = false;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    let remaining = OUTPUT_LIMIT_BYTES.saturating_sub(output.len());
                    if remaining == 0 {
                        truncated = true;
                        continue;
                    }
                    let chunk = String::from_utf8_lossy(&buffer[..read]);
                    if chunk.len() > remaining {
                        output.push_str(&prefix_by_bytes(&chunk, remaining));
                        truncated = true;
                    } else {
                        output.push_str(&chunk);
                    }
                }
                Err(_) => break,
            }
        }
        if truncated {
            output.push_str(&truncated_marker());
        }
        let _ = output_tx.send((output, truncated));
    });

    let started = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            drop(pair.master);
            let (output, truncated) = output_rx
                .recv_timeout(OUTPUT_DRAIN_TIMEOUT)
                .unwrap_or_default();
            events.push(AgentEvent::Output {
                stream: "pty".to_string(),
                text: output,
                truncated,
            });
            events.push(AgentEvent::Completed {
                exit_code: Some(status.exit_code() as i32),
            });
            return Ok(events);
        }
        if started.elapsed() >= options.timeout {
            let _ = child.kill();
            let _ = child.wait();
            drop(pair.master);
            events.push(AgentEvent::TimedOut);
            return Ok(events);
        }
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn run_adapter_process(
    config: &AdapterConfig,
    options: PtyRunOptions,
) -> Result<Vec<AgentEvent>> {
    let mut events = vec![AgentEvent::Started {
        adapter: options.adapter_name,
    }];
    let mut command = Command::new(&config.command);
    command.args(&config.args).arg(options.prompt);
    for (key, value) in &config.env {
        command.env(key, value);
    }
    if let Some(cwd) = &config.working_directory {
        command.current_dir(cwd);
    }

    match crate::adapter_probe_command::output_with_timeout(&mut command, options.timeout)
        .context("failed to run adapter process")?
    {
        Some(output) => {
            let mut text = String::new();
            text.push_str(&String::from_utf8_lossy(&output.stdout));
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            let (text, truncated) = truncate_output(text);
            events.push(AgentEvent::Output {
                stream: "process".to_string(),
                text,
                truncated,
            });
            events.push(AgentEvent::Completed {
                exit_code: output.status.code(),
            });
        }
        None => events.push(AgentEvent::TimedOut),
    }
    Ok(events)
}

fn truncate_output(text: String) -> (String, bool) {
    if text.len() <= OUTPUT_LIMIT_BYTES {
        return (text, false);
    }
    let mut output = prefix_by_bytes(&text, OUTPUT_LIMIT_BYTES);
    output.push_str(&truncated_marker());
    (output, true)
}

fn truncated_marker() -> String {
    format!("[truncated after {OUTPUT_LIMIT_BYTES} bytes]")
}

fn prefix_by_bytes(text: &str, max_bytes: usize) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if out.len() + ch.len_utf8() > max_bytes {
            break;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn pty_runner_reports_success() {
        let events = run_adapter_pty(
            &AdapterConfig {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "printf ok".to_string()],
                ..AdapterConfig::default()
            },
            PtyRunOptions {
                adapter_name: "shell".to_string(),
                prompt: String::new(),
                timeout: Duration::from_secs(2),
            },
        )
        .expect("pty run");

        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::Completed { .. }))
        );
        assert!(
            events.iter().any(
                |event| matches!(event, AgentEvent::Output { text, .. } if text.contains("ok"))
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn pty_runner_reports_timeout() {
        let events = run_adapter_pty(
            &AdapterConfig {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "sleep 1".to_string()],
                ..AdapterConfig::default()
            },
            PtyRunOptions {
                adapter_name: "shell".to_string(),
                prompt: String::new(),
                timeout: Duration::from_millis(10),
            },
        )
        .expect("pty run");

        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::TimedOut))
        );
    }

    #[cfg(unix)]
    #[test]
    fn pty_runner_marks_truncated_output() {
        let events = run_adapter_pty(
            &AdapterConfig {
                command: "sh".to_string(),
                args: vec![
                    "-c".to_string(),
                    format!(
                        "python3 - <<'PY'\nprint('x' * {})\nPY",
                        OUTPUT_LIMIT_BYTES + 1
                    ),
                ],
                ..AdapterConfig::default()
            },
            PtyRunOptions {
                adapter_name: "shell".to_string(),
                prompt: String::new(),
                timeout: Duration::from_secs(2),
            },
        )
        .expect("pty run");

        assert!(events.iter().any(|event| matches!(
            event,
            AgentEvent::Output {
                text,
                truncated: true,
                ..
            } if text.contains(&truncated_marker())
        )));
    }

    #[test]
    fn process_runner_reports_success_without_pty() {
        let events = run_adapter_process(
            &AdapterConfig {
                command: "node".to_string(),
                args: vec!["-e".to_string(), "process.stdout.write('ok')".to_string()],
                pty: false,
                ..AdapterConfig::default()
            },
            PtyRunOptions {
                adapter_name: "walkthrough".to_string(),
                prompt: String::new(),
                timeout: Duration::from_secs(2),
            },
        )
        .expect("process run");

        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::Completed { .. }))
        );
        assert!(
            events.iter().any(
                |event| matches!(event, AgentEvent::Output { text, .. } if text.contains("ok"))
            )
        );
    }
}
