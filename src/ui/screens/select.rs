use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{DemoScreen, FightScreen, MapScreen};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy)]
pub enum ScreenKind {
    Fight,
    Map,
    Demo,
}

impl ScreenKind {
    pub const ALL: &'static [ScreenKind] = &[ScreenKind::Fight, ScreenKind::Map, ScreenKind::Demo];

    pub fn label(&self) -> &'static str {
        match self {
            ScreenKind::Fight => "Fight Screen",
            ScreenKind::Map => "Map Screen",
            ScreenKind::Demo => "Transition Demo",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ScreenKind::Fight => "Turn-based combat with the crab and an enemy.",
            ScreenKind::Map => "Choose your path across the dungeon floor.",
            ScreenKind::Demo => "Preview every node transition animation.",
        }
    }

    pub fn build(&self, player: &Player) -> Screen {
        match self {
            ScreenKind::Fight => Screen::Fight(FightScreen::new(player)),
            ScreenKind::Map => Screen::Map(MapScreen::new()),
            ScreenKind::Demo => Screen::Demo(DemoScreen::new()),
        }
    }
}

pub struct SelectScreen {
    pub selected: usize,
}

impl SelectScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        let len = ScreenKind::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => Transition::Quit,
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + len - 1) % len;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % len;
                Transition::Stay
            }
            KeyCode::Enter => Transition::Goto(ScreenKind::ALL[self.selected].build(player)),
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
                Constraint::Min(1),
                Constraint::Length(menu_height(ScreenKind::ALL.len())),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(area);

        render_title(frame, chunks[0]);
        render_menu(frame, chunks[1], self.selected);
        render_hint(frame, chunks[2]);
    }
}

fn menu_height(entries: usize) -> u16 {
    (entries as u16) * 3 + 2
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title_lines = vec![
        Line::from(Span::styled(
            "Caligo",
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "— WIP screens —",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let widget = Paragraph::new(title_lines).alignment(Alignment::Center);
    let inner = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(4),
        width: area.width,
        height: 3.min(area.height),
    };
    frame.render_widget(widget, inner);
}

fn render_menu(frame: &mut Frame, area: Rect, selected: usize) {
    let menu_width = 50.min(area.width);
    let menu_x = area.x + area.width.saturating_sub(menu_width) / 2;
    let menu_area = Rect {
        x: menu_x,
        y: area.y,
        width: menu_width,
        height: area.height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Select a screen ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, kind) in ScreenKind::ALL.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let label_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let desc_style = Style::default().fg(Color::DarkGray);

        lines.push(Line::from(vec![
            Span::styled(cursor, label_style),
            Span::styled(kind.label(), label_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(kind.description(), desc_style),
        ]));
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::styled(" navigate   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::styled(" select   ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::styled(" quit", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
