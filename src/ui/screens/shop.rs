use crate::data::enemies::{self, EnemyTier};
use crate::data::starters::Starter;
use crate::meta::{self, MonsterId, Upgrade};
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

/// Sub-states inside `ShopMode::Upgrades`. The player picks which
/// monster to invest in, then drills into that monster's upgrade list.
#[derive(Debug, Clone, PartialEq, Eq)]
enum UpgradeStage {
    PickMonster,
    UpgradeList { monster_id: MonsterId },
}

/// Between-runs shop. Two modes accessible via Tab:
///   * Upgrades — staged: pick which owned monster, then buy ranks.
///   * Recruits — buy captured monsters from `Meta.pending_captures`.
pub struct ShopScreen {
    mode: ShopMode,
    /// Sub-state for the Upgrades mode.
    upgrade_stage: UpgradeStage,
    /// Index into the sorted owned-monster list for the picker.
    monster_idx: usize,
    /// Index into `Upgrade::ALL` for the highlighted upgrade row.
    upgrade_selected: usize,
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
            upgrade_stage: UpgradeStage::PickMonster,
            monster_idx: 0,
            upgrade_selected: 0,
            recruit_idx: 0,
            message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        // Mode toggle: Tab flips Upgrades <-> Recruits in either mode.
        // Switching back to Upgrades always lands on the picker stage.
        if matches!(key, KeyCode::Tab | KeyCode::BackTab) {
            self.mode = match self.mode {
                ShopMode::Upgrades => ShopMode::Recruits,
                ShopMode::Recruits => ShopMode::Upgrades,
            };
            if matches!(self.mode, ShopMode::Upgrades) {
                self.upgrade_stage = UpgradeStage::PickMonster;
            }
            self.message = None;
            return Transition::Stay;
        }
        match self.mode {
            ShopMode::Upgrades => self.handle_upgrades(key),
            ShopMode::Recruits => self.handle_recruits(key),
        }
    }

    fn handle_upgrades(&mut self, key: KeyCode) -> Transition {
        match self.upgrade_stage.clone() {
            UpgradeStage::PickMonster => self.handle_pick_monster(key),
            UpgradeStage::UpgradeList { monster_id } => {
                self.handle_upgrade_list(key, &monster_id)
            }
        }
    }

    fn handle_pick_monster(&mut self, key: KeyCode) -> Transition {
        let owned = sorted_owned_ids();
        let len = owned.len();
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                Transition::Goto(Screen::Start(StartScreen::new()))
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if len > 0 {
                    self.monster_idx = (self.monster_idx + len - 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 {
                    self.monster_idx = (self.monster_idx + 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Enter => {
                if let Some(id) = owned.get(self.monster_idx) {
                    self.upgrade_stage = UpgradeStage::UpgradeList {
                        monster_id: id.clone(),
                    };
                    self.upgrade_selected = 0;
                    self.message = None;
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn handle_upgrade_list(&mut self, key: KeyCode, monster_id: &MonsterId) -> Transition {
        let len = Upgrade::ALL.len();
        match key {
            // Esc backs out to the picker rather than leaving the shop.
            KeyCode::Esc | KeyCode::Char('q') => {
                self.upgrade_stage = UpgradeStage::PickMonster;
                self.message = None;
                Transition::Stay
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if len > 0 {
                    self.upgrade_selected = (self.upgrade_selected + len - 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 {
                    self.upgrade_selected = (self.upgrade_selected + 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Enter => {
                self.try_buy_upgrade(monster_id);
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

    fn try_buy_upgrade(&mut self, monster_id: &MonsterId) {
        let Some(upgrade) = Upgrade::ALL.get(self.upgrade_selected).copied() else {
            return;
        };
        let snap = meta::snapshot();
        let species = snap
            .monsters
            .get(monster_id)
            .map(|m| m.species.clone())
            .unwrap_or_else(|| "monster".to_string());
        let ranks = meta::ranks_for(monster_id);
        let current = upgrade.current_rank(&ranks);
        let Some(cost) = upgrade.cost_for_next(current) else {
            self.message = Some(format!(
                "{} is maxed for {}.",
                upgrade.name(),
                species
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
        if meta::try_buy(upgrade, monster_id) {
            self.message = Some(format!(
                "{} \u{2192} rank {} for {}.",
                upgrade.name(),
                current + 1,
                species
            ));
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
        let active_template = self.active_upgrade_template();
        render_mode_header(
            frame,
            self.mode,
            &self.upgrade_stage,
            active_template.as_ref(),
            chunks[1],
        );
        match self.mode {
            ShopMode::Upgrades => match &self.upgrade_stage {
                UpgradeStage::PickMonster => {
                    render_monster_picker(frame, self.monster_idx, chunks[3]);
                }
                UpgradeStage::UpgradeList { monster_id } => {
                    render_upgrades(frame, monster_id, self.upgrade_selected, chunks[3]);
                }
            },
            ShopMode::Recruits => {
                render_recruits(frame, self.recruit_idx, chunks[3]);
            }
        }
        if let Some(msg) = self.message.as_deref() {
            render_message(frame, msg, chunks[4]);
        }
        render_hint(frame, self.mode, &self.upgrade_stage, chunks[5]);
    }

    /// When the user is buying upgrades for a specific monster, return a
    /// `Starter`-shaped template for it (synthesised for wild captures)
    /// so the header can render its name in its type color.
    fn active_upgrade_template(&self) -> Option<Starter> {
        let UpgradeStage::UpgradeList { monster_id } = &self.upgrade_stage else {
            return None;
        };
        let snap = meta::snapshot();
        let instance = snap.monsters.get(monster_id)?;
        crate::run::build_party_member_from_instance(instance).map(|m| m.template)
    }
}

fn sorted_owned_ids() -> Vec<MonsterId> {
    let snap = meta::snapshot();
    let mut ids: Vec<MonsterId> = snap.monsters.keys().cloned().collect();
    ids.sort();
    ids
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
    stage: &UpgradeStage,
    template: Option<&Starter>,
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
        if let UpgradeStage::UpgradeList { .. } = stage {
            if let Some(template) = template {
                spans.extend([
                    Span::styled("   \u{00B7}   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("\u{2192} {}", template.name),
                        Style::default()
                            .fg(template.primary_type.color())
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);
            }
        }
    }
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_monster_picker(frame: &mut Frame, selected: usize, area: Rect) {
    let snap = meta::snapshot();
    let owned = sorted_owned_ids();
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
            " Pick a monster to upgrade ",
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
        let species = snap
            .monsters
            .get(id)
            .map(|m| m.species.clone())
            .unwrap_or_else(|| id.clone());
        let ranks = snap.monster_ranks.get(id).copied().unwrap_or_default();
        let total =
            ranks.tidepool + ranks.wellspring + ranks.quickfoot + ranks.sharpened_edge;
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let in_party = snap.party.iter().any(|p| p == id);
        let suffix = if in_party { "  (in party)" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(format!("{:<22}", species), style),
            Span::styled(
                format!("ranks {}", total),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(suffix.to_string(), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
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

/// Per-tier recruit price. With wild captures going straight into the
/// collection, the post-run shop only sells starter recruits in
/// practice — those are priced at a flat 10 embers each (the bestiary
/// tiers are kept around for future "shop also stocks captures" knobs).
pub fn recruit_price(species: &str) -> u32 {
    match enemies::tier_for_species(species) {
        Some(EnemyTier::Easy) => 20,
        Some(EnemyTier::Normal) => 50,
        Some(EnemyTier::Elite) => 120,
        Some(EnemyTier::Boss) => 300,
        None => 10,
    }
}

fn render_upgrades(
    frame: &mut Frame,
    monster_id: &MonsterId,
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
    let ranks = meta::ranks_for(monster_id);

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

fn render_hint(
    frame: &mut Frame,
    mode: ShopMode,
    stage: &UpgradeStage,
    area: Rect,
) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let mut spans = vec![
        Span::styled("Tab", key),
        Span::styled(" mode   ", dim),
        Span::styled("\u{2191}\u{2193}", key),
        Span::styled(" navigate   ", dim),
    ];
    let (action_label, esc_label) = match (mode, stage) {
        (ShopMode::Upgrades, UpgradeStage::PickMonster) => ("pick", "back to menu"),
        (ShopMode::Upgrades, UpgradeStage::UpgradeList { .. }) => ("buy", "monsters"),
        (ShopMode::Recruits, _) => ("buy", "back to menu"),
    };
    spans.extend([
        Span::styled("Enter", key),
        Span::styled(format!(" {}   ", action_label), dim),
        Span::styled("Esc", key),
        Span::styled(format!(" {}", esc_label), dim),
    ]);
    let hint = Line::from(spans);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
