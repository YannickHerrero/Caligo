use crate::meta;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::StartScreen;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Cross-run roster manager. Lists every owned monster, lets the player
/// toggle which ones are in the active party (max 6), and shows a quick
/// summary of each monster's permanent rank investment.
pub struct CollectionScreen {
    selected: usize,
    message: Option<String>,
}

impl CollectionScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        let snap = meta::snapshot();
        let owned = sorted_owned_ids(&snap);
        let len = owned.len();
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                Transition::Goto(Screen::Start(StartScreen::new()))
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if len > 0 {
                    self.selected = (self.selected + len - 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 {
                    self.selected = (self.selected + 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(id) = owned.get(self.selected) {
                    match meta::toggle_party(id) {
                        Ok(_) => self.message = None,
                        Err(e) => self.message = Some(e),
                    }
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
        if area.width == 0 || area.height == 0 {
            return;
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(1), // party count + cap
                Constraint::Length(1), // spacer
                Constraint::Min(8),    // owned list
                Constraint::Length(1), // message
                Constraint::Length(1), // hint
            ])
            .split(area);

        let snap = meta::snapshot();
        render_title(frame, chunks[0]);
        render_party_count(frame, &snap, chunks[1]);
        render_owned_list(frame, &snap, self.selected, chunks[3]);
        if let Some(msg) = self.message.as_deref() {
            render_message(frame, msg, chunks[4]);
        }
        render_hint(frame, chunks[5]);
    }
}

fn sorted_owned_ids(snap: &meta::Meta) -> Vec<meta::MonsterId> {
    let mut ids: Vec<meta::MonsterId> = snap.monsters.keys().cloned().collect();
    ids.sort();
    ids
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Caligo \u{2014} Collection",
        Style::default()
            .fg(Color::Rgb(255, 140, 90))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_party_count(frame: &mut Frame, snap: &meta::Meta, area: Rect) {
    let line = Line::from(vec![
        Span::styled("Party ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}/{}", snap.party.len(), meta::PARTY_CAP),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   \u{00B7}   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Owned ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", snap.monsters.len()),
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_owned_list(frame: &mut Frame, snap: &meta::Meta, selected: usize, area: Rect) {
    let owned = sorted_owned_ids(snap);
    let panel_w = 64.min(area.width.saturating_sub(2)).max(40);
    let needed_h = (owned.len().max(1) as u16) * 2 + 2;
    let panel_h = needed_h.min(area.height);
    let panel = Rect {
        x: area.x + (area.width.saturating_sub(panel_w)) / 2,
        y: area.y + (area.height.saturating_sub(panel_h)) / 2,
        width: panel_w,
        height: panel_h,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Owned Monsters ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);
    if inner.height == 0 {
        return;
    }

    if owned.is_empty() {
        let line = Line::from(Span::styled(
            "(no monsters owned yet)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (idx, id) in owned.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "\u{25B6} " } else { "  " };
        let in_party = snap.party.iter().any(|p| p == id);
        let mark = if in_party { "[\u{2713}]" } else { "[ ]" };
        let header_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let mark_style = if in_party {
            Style::default()
                .fg(Color::Rgb(180, 220, 130))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let species = snap
            .monsters
            .get(id)
            .map(|m| m.species.clone())
            .unwrap_or_else(|| id.clone());
        // Brief rank summary (sum of all ranks across the four ladders).
        let ranks = snap.monster_ranks.get(id).copied().unwrap_or_default();
        let total_ranks =
            ranks.tidepool + ranks.wellspring + ranks.quickfoot + ranks.sharpened_edge;
        lines.push(Line::from(vec![
            Span::styled(cursor, header_style),
            Span::styled(format!("{} ", mark), mark_style),
            Span::styled(format!("{:<22}", species), header_style),
            Span::styled(
                format!("ranks {}", total_ranks),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_message(frame: &mut Frame, message: &str, area: Rect) {
    let line = Line::from(Span::styled(
        message.to_string(),
        Style::default().fg(Color::Rgb(255, 210, 110)),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("\u{2191}\u{2193}", key),
        Span::styled(" navigate   ", dim),
        Span::styled("Enter", key),
        Span::styled(" toggle party   ", dim),
        Span::styled("Esc", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
