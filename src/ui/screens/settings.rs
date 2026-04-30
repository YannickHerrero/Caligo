use crate::player::Player;
use crate::settings;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{SelectScreen, StartScreen};
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy)]
pub enum SettingsOrigin {
    Select,
    Start,
}

impl SettingsOrigin {
    fn back(self) -> Screen {
        match self {
            SettingsOrigin::Select => Screen::Select(SelectScreen::new()),
            SettingsOrigin::Start => Screen::Start(StartScreen::new()),
        }
    }
}
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy)]
enum SettingItem {
    Theme,
}

impl SettingItem {
    const ALL: &'static [SettingItem] = &[SettingItem::Theme];

    fn label(&self) -> &'static str {
        match self {
            SettingItem::Theme => "Theme",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            SettingItem::Theme => {
                "Color palette tuned for a light or dark terminal background."
            }
        }
    }

    fn current_value(&self) -> &'static str {
        match self {
            SettingItem::Theme => settings::theme().label(),
        }
    }

    fn cycle(&self) {
        match self {
            SettingItem::Theme => settings::set_theme(settings::theme().toggle()),
        }
    }
}

pub struct SettingsScreen {
    pub selected: usize,
    origin: SettingsOrigin,
}

impl SettingsScreen {
    pub fn new(origin: SettingsOrigin) -> Self {
        Self {
            selected: 0,
            origin,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        let len = SettingItem::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => Transition::Goto(self.origin.back()),
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + len - 1) % len;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % len;
                Transition::Stay
            }
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l')
            | KeyCode::Enter => {
                if let Some(item) = SettingItem::ALL.get(self.selected) {
                    item.cycle();
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
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
    let line = Line::from(vec![Span::styled(
        "Caligo — Settings",
        Style::default()
            .fg(Color::Rgb(255, 140, 90))
            .add_modifier(Modifier::BOLD),
    )]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_list(frame: &mut Frame, selected: usize, area: Rect) {
    let list_w = 56.min(area.width.saturating_sub(2)).max(20);
    let list_h = ((SettingItem::ALL.len() as u16) * 3 + 2).min(area.height);
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
            " Settings ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(list_area);
    frame.render_widget(block, list_area);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let label_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let value_style = Style::default()
            .fg(Color::Rgb(120, 200, 255))
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::DarkGray);

        lines.push(Line::from(vec![
            Span::styled(cursor, label_style),
            Span::styled(format!("{:<14}", item.label()), label_style),
            Span::styled(item.current_value().to_string(), value_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(item.description(), desc_style),
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
        Span::styled("← →", key),
        Span::styled(" / ", dim),
        Span::styled("Enter", key),
        Span::styled(" change   ", dim),
        Span::styled("q", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
