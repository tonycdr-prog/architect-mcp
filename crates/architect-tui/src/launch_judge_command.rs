use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgeCommandEvidence {
    pub command: String,
    pub attempted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    pub exit_code: Option<i32>,
    pub stdout_tail: Vec<String>,
    pub stderr_tail: Vec<String>,
    pub error: Option<String>,
}

pub(crate) fn skipped_command(command: &str) -> LaunchJudgeCommandEvidence {
    LaunchJudgeCommandEvidence {
        command: command.to_string(),
        attempted: false,
        ok: None,
        exit_code: None,
        stdout_tail: Vec::new(),
        stderr_tail: Vec::new(),
        error: None,
    }
}

pub(crate) fn tail_lines(text: &str, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
    if lines.len() > max {
        lines = lines.split_off(lines.len() - max);
    }
    lines
        .into_iter()
        .map(|line| truncate_for_report(&line, 800))
        .collect()
}

fn truncate_for_report(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("[truncated]");
    truncated
}
