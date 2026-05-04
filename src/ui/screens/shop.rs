use crate::meta::{self, Upgrade};
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

/// Between-runs shop where the player spends embers on permanent
/// stat-ladder ranks.
pub struct ShopScreen {
    selected: usize,
    /// Transient one-line message shown under the menu (e.g. "Not enough
    /// embers!" or "Already at max rank."). Cleared on next nav.
    message: Option<String>,
}

impl ShopScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        let len = Upgrade::ALL.len();
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
            KeyCode::Enter => {
                self.try_buy();
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
                Constraint::Length(1), // ember balance
                Constraint::Length(1), // spacer
                Constraint::Min(8),    // upgrade list
                Constraint::Length(1), // message strip
                Constraint::Length(1), // hint
            ])
            .split(area);

        render_title(frame, chunks[0]);
        render_balance(frame, chunks[1]);
        render_upgrades(frame, self.selected, chunks[3]);
        if let Some(msg) = self.message.as_deref() {
            render_message(frame, msg, chunks[4]);
        }
        render_hint(frame, chunks[5]);
    }

    fn try_buy(&mut self) {
        let Some(upgrade) = Upgrade::ALL.get(self.selected).copied() else {
            return;
        };
        let snap = meta::snapshot();
        let current = upgrade.current_rank(&snap);
        let Some(cost) = upgrade.cost_for_next(current) else {
            self.message = Some(format!("{} is already maxed.", upgrade.name()));
            return;
        };
        if snap.embers < cost {
            self.message = Some(format!(
                "Need {} embers ({} short).",
                cost,
                cost - snap.embers
            ));
            return;
        }
        if meta::try_buy(upgrade) {
            self.message = Some(format!(
                "{} \u{2192} rank {}.",
                upgrade.name(),
                current + 1
            ));
        }
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Caligo \u{2014} Ember Shop",
        Style::default()
            .fg(Color::Rgb(255, 140, 90))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_balance(frame: &mut Frame, area: Rect) {
    let snap = meta::snapshot();
    let line = Line::from(vec![
        Span::styled("Embers ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", snap.embers),
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_upgrades(frame: &mut Frame, selected: usize, area: Rect) {
    let panel_w = 64.min(area.width.saturating_sub(2)).max(40);
    let needed_h = (Upgrade::ALL.len() as u16) * 3 + 2;
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
            " Permanent Upgrades ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);
    if inner.height == 0 {
        return;
    }

    let snap = meta::snapshot();
    let mut lines: Vec<Line> = Vec::new();
    for (idx, upgrade) in Upgrade::ALL.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "\u{25B6} " } else { "  " };
        let current = upgrade.current_rank(&snap);
        let max = upgrade.max_rank();
        let cost_text = match upgrade.cost_for_next(current) {
            Some(c) => format!("{} embers", c),
            None => "MAX".to_string(),
        };

        let header_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let cost_style = if upgrade.cost_for_next(current).is_none() {
            Style::default()
                .fg(Color::Rgb(120, 200, 120))
                .add_modifier(Modifier::BOLD)
        } else if upgrade.cost_for_next(current).map_or(false, |c| snap.embers >= c) {
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(cursor, header_style),
            Span::styled(format!("{:<22}", upgrade.name()), header_style),
            Span::styled(
                format!("rank {}/{}  ", current, max),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(cost_text, cost_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                upgrade.description(),
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
        Span::styled(" buy   ", dim),
        Span::styled("Esc", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
