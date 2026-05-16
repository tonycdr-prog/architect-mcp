use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};

use super::app::{AppState, Panel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DirtyRegion {
    Sessions,
    Transcript,
    Inspector,
    Command,
    Full,
}

#[derive(Debug, Clone)]
pub struct RenderScheduler {
    dirty: BTreeSet<DirtyRegion>,
    last_draw: Instant,
    frame_budget: Duration,
}

impl RenderScheduler {
    pub fn new(frame_budget: Duration) -> Self {
        Self {
            dirty: BTreeSet::from([DirtyRegion::Full]),
            last_draw: Instant::now() - frame_budget,
            frame_budget,
        }
    }

    pub fn mark(&mut self, region: DirtyRegion) {
        if region == DirtyRegion::Full {
            self.dirty.clear();
        }
        self.dirty.insert(region);
    }

    pub fn mark_panels(&mut self, panels: BTreeSet<Panel>) {
        for panel in panels {
            self.mark(match panel {
                Panel::Sessions => DirtyRegion::Sessions,
                Panel::Transcript => DirtyRegion::Transcript,
                Panel::Inspector => DirtyRegion::Inspector,
                Panel::Command => DirtyRegion::Command,
            });
        }
    }

    pub fn should_draw(&self) -> bool {
        !self.dirty.is_empty() && self.last_draw.elapsed() >= self.frame_budget
    }

    pub fn take_dirty(&mut self) -> BTreeSet<DirtyRegion> {
        self.last_draw = Instant::now();
        std::mem::take(&mut self.dirty)
    }
}

pub fn handle_event(app: &mut AppState, event: Event) -> bool {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Tab => app.next_panel(),
            KeyCode::Enter => app.submit_input(),
            KeyCode::Backspace => app.backspace(),
            KeyCode::Char(ch) => app.append_input(ch),
            _ => {}
        },
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(panel) = app.panel_at(mouse.column, mouse.row) {
                    app.active_panel = panel;
                    match panel {
                        Panel::Sessions => {
                            if let Some(index) = app.agent_index_at(mouse.row) {
                                app.pin_agent_at(index);
                            }
                        }
                        Panel::Transcript | Panel::Inspector | Panel::Command => {
                            app.mark_dirty(panel);
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let panel = app
                    .panel_at(mouse.column, mouse.row)
                    .unwrap_or(Panel::Transcript);
                if panel == Panel::Transcript {
                    app.transcript.push("scroll: transcript".to_string());
                    app.mark_dirty(Panel::Transcript);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if app.panel_at(mouse.column, mouse.row) == Some(Panel::Inspector) {
                    app.inspector
                        .push(format!("drag: {},{}", mouse.column, mouse.row));
                    app.mark_dirty(Panel::Inspector);
                }
            }
            _ => {}
        },
        Event::Resize(_, _) => {
            app.mark_dirty(Panel::Sessions);
            app.mark_dirty(Panel::Transcript);
            app.mark_dirty(Panel::Inspector);
            app.mark_dirty(Panel::Command);
        }
        Event::FocusGained | Event::FocusLost => {
            app.mark_dirty(app.active_panel);
        }
        _ => {}
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TuiConfig;
    use crossterm::event::{KeyEvent, KeyModifiers, MouseEvent};
    use ratatui::layout::Rect;

    #[test]
    fn key_events_update_input_and_submit() {
        let mut app = AppState::new(".".into(), TuiConfig::default());
        assert!(!handle_event(
            &mut app,
            Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE))
        ));
        assert_eq!(app.input, "h");
        handle_event(
            &mut app,
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );
        assert!(app.input.is_empty());
        assert!(app.transcript.iter().any(|line| line == "user: h"));
    }

    #[test]
    fn mouse_click_pins_agent_and_drag_updates_inspector() {
        let mut app = AppState::new(".".into(), TuiConfig::default());
        app.set_layout(super::super::app::PanelLayout::from_area(Rect::new(
            0, 0, 120, 30,
        )));
        handle_event(
            &mut app,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 2,
                row: 1,
                modifiers: KeyModifiers::NONE,
            }),
        );
        assert!(app.agents[0].pinned);
        handle_event(
            &mut app,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 95,
                row: 9,
                modifiers: KeyModifiers::NONE,
            }),
        );
        assert!(app.inspector.iter().any(|line| line.contains("drag")));
    }

    #[test]
    fn scheduler_coalesces_dirty_regions() {
        let mut scheduler = RenderScheduler::new(Duration::from_millis(0));
        scheduler.mark(DirtyRegion::Command);
        scheduler.mark(DirtyRegion::Command);
        assert!(scheduler.should_draw());
        let dirty = scheduler.take_dirty();
        assert!(dirty.contains(&DirtyRegion::Full) || dirty.contains(&DirtyRegion::Command));
        assert!(!scheduler.should_draw());
    }
}
