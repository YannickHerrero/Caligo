use crate::data::enemies::{self, EnemyTier};
use crate::data::starters::{self, Starter};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShopMode {
    /// Spend embers on permanent stat ladders for an owned monster.
    Upgrades,
    /// Spend embers to add captured monsters to your permanent collection.
    Recruits,
}

/// Between-runs shop. Two modes accessible via Tab:
///   * Upgrades — permanent stat ranks per owned monster (Left/Right
///     cycles which monster, Up/Down picks the upgrade row).
///   * Recruits — buy captured monsters from `Meta.pending_captures`.
pub struct ShopScreen {
    mode: ShopMode,
    starters: Vec<Starter>,
    /// Index into `starters` for the active monster in Upgrades mode.
    starter_idx: usize,
    /// Index into `Upgrade::ALL` for the highlighted upgrade row.
    selected: usize,
    /// Index into the pending-captures vec for the highlighted recruit row.
    recruit_idx: usize,
    /// Transient one-line message shown under the menu (e.g. "Not enough
    /// embers!" or "Already at max rank."). Cleared on next nav.
    message: Option<String>,
}

impl ShopScreen {
    pub fn new() -> Self {
        Self {
            mode: ShopMode::Upgrades,
            starters: starters::all_starters(),
            starter_idx: 0,
            selected: 0,
            recruit_idx: 0,
            message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        // Mode toggle: Tab flips Upgrades <-> Recruits in either mode.
        if matches!(key, KeyCode::Tab | KeyCode::BackTab) {
            self.mode = match self.mode {
                ShopMode::Upgrades => ShopMode::Recruits,
                ShopMode::Recruits => ShopMode::Upgrades,
            };
            self.message = None;
            return Transition::Stay;
        }
        match self.mode {
            ShopMode::Upgrades => self.handle_upgrades(key),
            ShopMode::Recruits => self.handle_recruits(key),
        }
    }

    fn handle_upgrades(&mut self, key: KeyCode) -> Transition {
        let len = Upgrade::ALL.len();
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                Transition::Goto(Screen::Start(StartScreen::new()))
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !self.starters.is_empty() {
                    self.starter_idx = (self.starter_idx + 1) % self.starters.len();
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if !self.starters.is_empty() {
                    let n = self.starters.len();
                    self.starter_idx = (self.starter_idx + n - 1) % n;
                }
                self.message = None;
                Transition::Stay
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
                self.try_buy_upgrade();
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn handle_recruits(&mut self, key: KeyCode) -> Transition {
        let snap = meta::snapshot();
        let len = snap.pending_captures.len();
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                Transition::Goto(Screen::Start(StartScreen::new()))
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if len > 0 {
                    self.recruit_idx = (self.recruit_idx + len - 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 {
                    self.recruit_idx = (self.recruit_idx + 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Enter => {
                self.try_buy_recruit();
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn try_buy_recruit(&mut self) {
        let snap = meta::snapshot();
        if self.recruit_idx >= snap.pending_captures.len() {
            return;
        }
        let recruit = snap.pending_captures[self.recruit_idx].clone();
        let price = recruit_price(&recruit.species);
        if snap.embers < price {
            self.message = Some(format!(
                "Need {} embers ({} short).",
                price,
                price - snap.embers
            ));
            return;
        }
        if meta::try_buy_recruit(self.recruit_idx, price) {
            self.message = Some(format!(
                "Recruited {} for {} embers.",
                recruit.species, price
            ));
            // Snap selection back into range.
            let new_len = meta::snapshot().pending_captures.len();
            if self.recruit_idx >= new_len {
                self.recruit_idx = new_len.saturating_sub(1);
            }
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
                Constraint::Length(1), // mode tabs + ember balance
                Constraint::Length(1), // spacer
                Constraint::Min(8),    // body
                Constraint::Length(1), // message strip
                Constraint::Length(1), // hint
            ])
            .split(area);

        render_title(frame, chunks[0]);
        render_mode_header(frame, self.mode, self.active_starter(), chunks[1]);
        match self.mode {
            ShopMode::Upgrades => {
                render_upgrades(frame, self.active_starter(), self.selected, chunks[3]);
            }
            ShopMode::Recruits => {
                render_recruits(frame, self.recruit_idx, chunks[3]);
            }
        }
        if let Some(msg) = self.message.as_deref() {
            render_message(frame, msg, chunks[4]);
        }
        render_hint(frame, self.mode, chunks[5]);
    }

    fn active_starter(&self) -> Option<&Starter> {
        self.starters.get(self.starter_idx)
    }

    fn try_buy_upgrade(&mut self) {
        let Some(upgrade) = Upgrade::ALL.get(self.selected).copied() else {
            return;
        };
        let Some(starter) = self.active_starter().cloned() else {
            return;
        };
        let monster_id = meta::starter_id(&starter.name);
        let snap = meta::snapshot();
        let ranks = meta::ranks_for(&monster_id);
        let current = upgrade.current_rank(&ranks);
        let Some(cost) = upgrade.cost_for_next(current) else {
            self.message = Some(format!(
                "{} is maxed for {}.",
                upgrade.name(),
                starter.name
            ));
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
        if meta::try_buy(upgrade, &monster_id) {
            self.message = Some(format!(
                "{} \u{2192} rank {} for {}.",
                upgrade.name(),
                current + 1,
                starter.name
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

fn render_mode_header(
    frame: &mut Frame,
    mode: ShopMode,
    starter: Option<&Starter>,
    area: Rect,
) {
    let snap = meta::snapshot();
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);
    let pending_count = snap.pending_captures.len();
    let upgrades_label = if mode == ShopMode::Upgrades {
        "[ Upgrades ]".to_string()
    } else {
        "  Upgrades  ".to_string()
    };
    let recruits_label = if mode == ShopMode::Recruits {
        format!("[ Recruits ({}) ]", pending_count)
    } else {
        format!("  Recruits ({})  ", pending_count)
    };
    let mut spans = vec![
        Span::styled("Embers ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", snap.embers),
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   \u{00B7}   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            upgrades_label,
            if mode == ShopMode::Upgrades {
                active_style
            } else {
                inactive_style
            },
        ),
        Span::raw(" "),
        Span::styled(
            recruits_label,
            if mode == ShopMode::Recruits {
                active_style
            } else {
                inactive_style
            },
        ),
    ];
    if mode == ShopMode::Upgrades {
        if let Some(starter) = starter {
            spans.extend([
                Span::styled("   \u{00B7}   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("\u{25C0} {} \u{25B6}", starter.name),
                    Style::default()
                        .fg(starter.primary_type.color())
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
        }
    }
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_recruits(frame: &mut Frame, selected: usize, area: Rect) {
    let snap = meta::snapshot();
    let panel_w = 64.min(area.width.saturating_sub(2)).max(40);
    let needed_h = (snap.pending_captures.len().max(1) as u16) * 2 + 2;
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
            " Available Recruits ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);
    if inner.height == 0 {
        return;
    }

    if snap.pending_captures.is_empty() {
        let line = Line::from(Span::styled(
            "(no pending captures \u{2014} catch some monsters in a run!)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (idx, recruit) in snap.pending_captures.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "\u{25B6} " } else { "  " };
        let price = recruit_price(&recruit.species);
        let header_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let cost_style = if snap.embers >= price {
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, header_style),
            Span::styled(format!("{:<22}", recruit.species), header_style),
            Span::styled(format!("{} embers", price), cost_style),
        ]));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Per-tier recruit price. Starters (not in the bestiary) default to a
/// flat 100 — they enter the pool via Phase 7's boss-kill drops.
pub fn recruit_price(species: &str) -> u32 {
    match enemies::tier_for_species(species) {
        Some(EnemyTier::Easy) => 20,
        Some(EnemyTier::Normal) => 50,
        Some(EnemyTier::Elite) => 120,
        Some(EnemyTier::Boss) => 300,
        None => 100,
    }
}

fn render_upgrades(
    frame: &mut Frame,
    starter: Option<&Starter>,
    selected: usize,
    area: Rect,
) {
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
    let ranks = match starter {
        Some(s) => meta::ranks_for(&meta::starter_id(&s.name)),
        None => Default::default(),
    };

    let mut lines: Vec<Line> = Vec::new();
    for (idx, upgrade) in Upgrade::ALL.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "\u{25B6} " } else { "  " };
        let current = upgrade.current_rank(&ranks);
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
        } else if upgrade
            .cost_for_next(current)
            .map_or(false, |c| snap.embers >= c)
        {
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

fn render_hint(frame: &mut Frame, mode: ShopMode, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let mut spans = vec![
        Span::styled("Tab", key),
        Span::styled(" mode   ", dim),
    ];
    match mode {
        ShopMode::Upgrades => {
            spans.extend([
                Span::styled("\u{2190}\u{2192}", key),
                Span::styled(" monster   ", dim),
                Span::styled("\u{2191}\u{2193}", key),
                Span::styled(" upgrade   ", dim),
            ]);
        }
        ShopMode::Recruits => {
            spans.extend([
                Span::styled("\u{2191}\u{2193}", key),
                Span::styled(" recruit   ", dim),
            ]);
        }
    }
    spans.extend([
        Span::styled("Enter", key),
        Span::styled(" buy   ", dim),
        Span::styled("Esc", key),
        Span::styled(" back", dim),
    ]);
    let hint = Line::from(spans);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
