use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{SelectScreen, TransitionKind, TransitionScreen};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct DemoScreen {
    pub selected: usize,
}

impl DemoScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    fn with_selected(selected: usize) -> Self {
        Self { selected }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Transition {
        let len = TransitionKind::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                Transition::Goto(Screen::Select(SelectScreen::new()))
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + len - 1) % len;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % len;
                Transition::Stay
            }
            KeyCode::Enter => {
                let kind = TransitionKind::ALL[self.selected];
                let from = std::mem::replace(self, DemoScreen::with_selected(self.selected));
                let to = DemoScreen::with_selected(self.selected);
                Transition::Goto(Screen::Transition(TransitionScreen::new(
                    Screen::Demo(from),
                    Screen::Demo(to),
                    kind,
                )))
            }
            _ => Transition::Stay,
        }
    }

    pub fn update(&mut self) -> Transition {
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .split(area);
        render_header(frame, chunks[0]);
        render_list(frame, self.selected, chunks[1]);
        render_hint(frame, chunks[2]);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            "Caligo — Transition demo",
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ·   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "press Enter to play",
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_list(frame: &mut Frame, selected: usize, area: Rect) {
    let list_w = 44.min(area.width.saturating_sub(2)).max(20);
    let list_h = ((TransitionKind::ALL.len() as u16) * 2 + 2).min(area.height);
    if list_w < 20 || list_h == 0 {
        return;
    }
    let list_area = Rect {
        x: area.x + (area.width.saturating_sub(list_w)) / 2,
        y: area.y + (area.height.saturating_sub(list_h)) / 2,
        width: list_w,
        height: list_h,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Transitions ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(list_area);
    frame.render_widget(block, list_area);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, kind) in TransitionKind::ALL.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let cursor_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let kind_style = Style::default()
            .fg(kind.color())
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(vec![
            Span::styled(cursor, cursor_style),
            Span::styled(kind.name(), kind_style),
        ]));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("↑ ↓", key),
        Span::styled(" navigate   ", dim),
        Span::styled("Enter", key),
        Span::styled(" play   ", dim),
        Span::styled("q", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
