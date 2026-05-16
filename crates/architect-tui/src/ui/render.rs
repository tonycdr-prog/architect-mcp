use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::config::TuiConfig;

use super::app::{AppState, Panel, PanelLayout};
use super::events::{RenderScheduler, handle_event};

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

pub async fn run_interactive(workspace: PathBuf, config: TuiConfig) -> Result<()> {
    let mut app = AppState::new(workspace, config.clone());
    let mut terminal = setup_terminal(config.ui.mouse)?;
    let mut scheduler = RenderScheduler::new(Duration::from_millis(config.ui.tick_millis));

    loop {
        scheduler.mark_panels(app.take_dirty());
        if scheduler.should_draw() {
            let _dirty = scheduler.take_dirty();
            terminal.draw(|frame| render_app(frame, &mut app))?;
        }
        if event::poll(Duration::from_millis(config.ui.tick_millis))? {
            let quit = handle_event(&mut app, event::read()?);
            scheduler.mark_panels(app.take_dirty());
            if quit {
                break;
            }
        }
    }

    restore_terminal(&mut terminal, config.ui.mouse)?;
    Ok(())
}

fn setup_terminal(mouse: bool) -> Result<TuiTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if mouse {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    } else {
        execute!(stdout, EnterAlternateScreen)?;
    }
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

fn restore_terminal(terminal: &mut TuiTerminal, mouse: bool) -> Result<()> {
    disable_raw_mode()?;
    if mouse {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
    } else {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    terminal.show_cursor()?;
    Ok(())
}

pub fn render_app(frame: &mut Frame<'_>, app: &mut AppState) {
    let layout = PanelLayout::from_area(frame.area());
    app.set_layout(layout);

    let agents: Vec<ListItem<'_>> = app
        .agents
        .iter()
        .map(|agent| {
            let pin = if agent.pinned { "*" } else { " " };
            ListItem::new(format!("{pin} {} [{}]", agent.name, agent.status))
        })
        .collect();
    frame.render_widget(
        List::new(agents).block(panel_block("Agents", app.active_panel == Panel::Sessions)),
        layout.sessions,
    );

    let transcript = app.transcript.join("\n");
    frame.render_widget(
        Paragraph::new(transcript)
            .block(panel_block(
                &app.title,
                app.active_panel == Panel::Transcript,
            ))
            .wrap(Wrap { trim: false }),
        layout.transcript,
    );

    let inspector = format!(
        "{}\n\nworkflow gates:\n{}",
        app.inspector.join("\n"),
        app.workflow
            .steps
            .iter()
            .map(|step| format!("- {} -> {}", step.name, step.gate))
            .collect::<Vec<_>>()
            .join("\n")
    );
    frame.render_widget(
        Paragraph::new(inspector)
            .block(panel_block(
                "Inspector",
                app.active_panel == Panel::Inspector,
            ))
            .wrap(Wrap { trim: false }),
        layout.inspector,
    );

    frame.render_widget(
        Paragraph::new(format!("> {}", app.input))
            .block(panel_block("Command", app.active_panel == Panel::Command)),
        layout.command,
    );
}

fn panel_block(title: &str, active: bool) -> Block<'_> {
    let style = if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TuiConfig;
    use ratatui::backend::TestBackend;

    #[test]
    fn renders_main_surfaces_to_test_backend() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = AppState::new(".".into(), TuiConfig::default());
        terminal
            .draw(|frame| render_app(frame, &mut app))
            .expect("draw");
        assert!(app.layout.is_some());
        let buffer = terminal.backend().buffer();
        let rendered = format!("{buffer:?}");
        assert!(rendered.contains("Agents"));
        assert!(rendered.contains("Inspector"));
        assert!(rendered.contains("Command"));
        assert!(rendered.contains("grill_me"));
    }
}
