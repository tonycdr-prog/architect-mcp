use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

use crate::adapter::{AdapterConfig, AgentEvent};

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
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    output.push_str(&String::from_utf8_lossy(&buffer[..read]));
                    if output.len() > 64 * 1024 {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = output_tx.send(output);
    });

    let started = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            drop(pair.master);
            let output = output_rx
                .recv_timeout(Duration::from_millis(200))
                .unwrap_or_default();
            events.push(AgentEvent::Output {
                stream: "pty".to_string(),
                text: output,
            });
            events.push(AgentEvent::Completed {
                exit_code: Some(status.exit_code() as i32),
            });
            return Ok(events);
        }
        if started.elapsed() >= options.timeout {
            let _ = child.kill();
            drop(pair.master);
            events.push(AgentEvent::TimedOut);
            return Ok(events);
        }
        thread::sleep(Duration::from_millis(20));
    }
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
}
