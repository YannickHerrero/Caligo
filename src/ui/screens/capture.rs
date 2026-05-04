use crate::fight::{Item, ItemStack};
use crate::map::NodeKind;
use crate::player::Player;
use crate::run::PartyMember;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{MapScreen, RewardScreen};
use crossterm::event::KeyCode;
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const ROLL_DURATION: f32 = 1.5;
const TICK_RATE: f32 = 0.05;

const PARTY_CAP: usize = 6;

/// Stages the capture flow walks through after a non-boss victory.
#[derive(Debug, Clone, PartialEq)]
enum CapturePhase {
    /// "Throw a Monster Net?" — Y / N.
    Confirm,
    /// Animation playing while we wait to reveal the result.
    Rolling { elapsed: f32 },
    /// Net succeeded — captured monster awaits party-slot decision.
    Caught,
    /// Net consumed but the monster escaped.
    Failed,
    /// Captured a 7th monster: ask which existing party slot to replace.
    PickReplaceSlot { selected: usize },
}

pub struct CapturePromptScreen {
    /// MapScreen carried forward; its `run` may have its `party`
    /// mutated when a capture lands.
    pub map: Option<Box<MapScreen>>,
    /// Reward args that will be handed to RewardScreen once the capture
    /// flow finishes.
    pub gold: u32,
    pub embers: u32,
    pub items: Vec<Item>,
    pub kind: NodeKind,
    /// The defeated enemy's species (capture target).
    pub enemy_species: String,
    /// Base catch rate for this enemy's tier.
    pub catch_rate: f32,
    phase: CapturePhase,
}

impl CapturePromptScreen {
    pub fn new(
        map: Box<MapScreen>,
        gold: u32,
        embers: u32,
        items: Vec<Item>,
        kind: NodeKind,
        enemy_species: String,
        catch_rate: f32,
    ) -> Self {
        Self {
            map: Some(map),
            gold,
            embers,
            items,
            kind,
            enemy_species,
            catch_rate,
            phase: CapturePhase::Confirm,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match self.phase.clone() {
            CapturePhase::Confirm => match key {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    consume_one_net(&mut player.inventory);
                    self.phase = CapturePhase::Rolling { elapsed: 0.0 };
                    Transition::Stay
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.exit_to_reward()
                }
                _ => Transition::Stay,
            },
            CapturePhase::Rolling { .. } => Transition::Stay,
            CapturePhase::Caught | CapturePhase::Failed => self.exit_to_reward(),
            CapturePhase::PickReplaceSlot { selected } => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    let n = self.party_len();
                    if n > 0 {
                        let new_sel = (selected + n - 1) % n;
                        self.phase = CapturePhase::PickReplaceSlot { selected: new_sel };
                    }
                    Transition::Stay
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let n = self.party_len();
                    if n > 0 {
                        let new_sel = (selected + 1) % n;
                        self.phase = CapturePhase::PickReplaceSlot { selected: new_sel };
                    }
                    Transition::Stay
                }
                KeyCode::Enter => {
                    self.replace_slot(selected);
                    self.exit_to_reward()
                }
                KeyCode::Esc => {
                    // Cancelling the slot pick releases the captured monster.
                    self.exit_to_reward()
                }
                _ => Transition::Stay,
            },
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        if let CapturePhase::Rolling { elapsed } = &mut self.phase {
            *elapsed += TICK_RATE;
            if *elapsed >= ROLL_DURATION {
                self.resolve_roll();
            }
        }
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Background: render the carried map, dimmed, so the player has
        // context. (RewardScreen is what's behind us logically; the map
        // is the next destination so it's a reasonable backdrop.)
        if let Some(map) = self.map.as_mut() {
            map.draw(frame, _player);
        }

        let popup_w: u16 = 56.min(area.width.saturating_sub(4)).max(20);
        let popup_h: u16 = match self.phase {
            CapturePhase::PickReplaceSlot { .. } => {
                (self.party_len() as u16 + 6).min(area.height.saturating_sub(2)).max(7)
            }
            _ => 9.min(area.height.saturating_sub(2)).max(5),
        };
        let popup = Rect {
            x: area.x + (area.width.saturating_sub(popup_w)) / 2,
            y: area.y + (area.height.saturating_sub(popup_h)) / 2,
            width: popup_w,
            height: popup_h,
        };

        // Dim everything outside the popup, then clear it.
        let buf = frame.buffer_mut();
        for y in area.y..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if x >= popup.x
                    && x < popup.x + popup.width
                    && y >= popup.y
                    && y < popup.y + popup.height
                {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((x, y)) {
                    let fg = match cell.fg {
                        Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                        other => other,
                    };
                    cell.set_fg(fg);
                }
            }
        }
        for y in popup.y..(popup.y + popup.height) {
            for x in popup.x..(popup.x + popup.width) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ').set_style(Style::default());
                }
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(160, 220, 200)))
            .title(Span::styled(
                " Monster Net ",
                Style::default()
                    .fg(Color::Rgb(160, 220, 200))
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(popup);
        frame.render_widget(block, popup);
        if inner.height == 0 {
            return;
        }

        match self.phase.clone() {
            CapturePhase::Confirm => self.draw_confirm(frame, inner),
            CapturePhase::Rolling { elapsed } => self.draw_rolling(frame, inner, elapsed),
            CapturePhase::Caught => self.draw_caught(frame, inner),
            CapturePhase::Failed => self.draw_failed(frame, inner),
            CapturePhase::PickReplaceSlot { selected } => {
                self.draw_pick_slot(frame, inner, selected);
            }
        }
    }

    fn draw_confirm(&self, frame: &mut Frame, inner: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Throw a net at the {}?", self.enemy_species),
                Style::default().fg(Color::Gray),
            )))
            .alignment(Alignment::Center),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Catch chance: ~{}%", (self.catch_rate * 100.0).round() as u32),
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center),
            chunks[1],
        );

        let key = Style::default().fg(Color::Yellow);
        let dim = Style::default().fg(Color::DarkGray);
        let prompt = Line::from(vec![
            Span::styled("Y", key),
            Span::styled(" / ", dim),
            Span::styled("Enter", key),
            Span::styled("  throw   ", dim),
            Span::styled("N", key),
            Span::styled(" / ", dim),
            Span::styled("Esc", key),
            Span::styled("  skip", dim),
        ]);
        frame.render_widget(
            Paragraph::new(prompt).alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn draw_rolling(&self, frame: &mut Frame, inner: Rect, elapsed: f32) {
        // Three-dot spinner driven by elapsed time.
        let dots = (((elapsed / 0.25) as usize) % 4).min(3);
        let mut dot_str = String::new();
        for _ in 0..dots {
            dot_str.push('.');
        }
        let line = Line::from(Span::styled(
            format!("Throwing net{}", dot_str),
            Style::default()
                .fg(Color::Rgb(160, 220, 200))
                .add_modifier(Modifier::BOLD),
        ));
        let mut top = inner;
        top.height = top.height.min(2);
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            top,
        );
    }

    fn draw_caught(&self, frame: &mut Frame, inner: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Caught {}!", self.enemy_species),
                Style::default()
                    .fg(Color::Rgb(180, 220, 130))
                    .add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "It's added to the run roster, and to the post-run shop.",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center),
            chunks[1],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Press any key to continue.",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn draw_failed(&self, frame: &mut Frame, inner: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("The {} got away.", self.enemy_species),
                Style::default().fg(Color::Rgb(220, 130, 130)),
            )))
            .alignment(Alignment::Center),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "The net was consumed.",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center),
            chunks[1],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Press any key to continue.",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn draw_pick_slot(&self, frame: &mut Frame, inner: Rect, selected: usize) {
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Party full \u{2014} replace which slot with {}?", self.enemy_species),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
        if let Some(map) = self.map.as_ref() {
            for (idx, member) in map.run.party.iter().enumerate() {
                let style = if idx == selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let cursor = if idx == selected { "\u{25B6} " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(cursor, style),
                    Span::styled(member.template.name.clone(), style),
                ]));
            }
        }
        lines.push(Line::from(""));
        let key = Style::default().fg(Color::Yellow);
        let dim = Style::default().fg(Color::DarkGray);
        lines.push(Line::from(vec![
            Span::styled("\u{2191}\u{2193}", key),
            Span::styled(" pick   ", dim),
            Span::styled("Enter", key),
            Span::styled(" replace   ", dim),
            Span::styled("Esc", key),
            Span::styled(" release new", dim),
        ]));
        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center),
            inner,
        );
    }

    fn resolve_roll(&mut self) {
        let mut rng = rand::thread_rng();
        let success = rng.gen_bool(self.catch_rate.clamp(0.0, 1.0) as f64);
        if !success {
            self.phase = CapturePhase::Failed;
            return;
        }

        // Mint the wild monster.
        let id = crate::meta::mint_wild_id();
        let instance = crate::meta::MonsterInstance {
            id: id.clone(),
            species: self.enemy_species.clone(),
        };
        crate::meta::push_pending_capture(instance.clone());

        // Add to the active run roster if there's room; otherwise pop the
        // replace-slot picker.
        if let Some(map) = self.map.as_mut() {
            if map.run.party.len() < PARTY_CAP {
                // Wild captures don't have a Starter template yet. For
                // Phase 4 we wrap the species in a placeholder Starter
                // sourced from the bestiary so the run can render and
                // act on it. A unified MonsterTemplate replaces this in
                // a follow-up.
                if let Some(member) = build_party_member_from_capture(&instance) {
                    map.run.party.push(member);
                    self.phase = CapturePhase::Caught;
                    return;
                }
            } else {
                self.phase = CapturePhase::PickReplaceSlot { selected: 0 };
                return;
            }
        }
        // Fallback: treat as caught even if we couldn't add to run party.
        self.phase = CapturePhase::Caught;
    }

    fn replace_slot(&mut self, slot: usize) {
        let id = crate::meta::mint_wild_id();
        let instance = crate::meta::MonsterInstance {
            id: id.clone(),
            species: self.enemy_species.clone(),
        };
        // pending_capture was already pushed in resolve_roll(); don't
        // double-push. Just swap the run slot.
        if let (Some(map), Some(member)) =
            (self.map.as_mut(), build_party_member_from_capture(&instance))
        {
            if slot < map.run.party.len() {
                map.run.party[slot] = member;
            }
        }
    }

    fn party_len(&self) -> usize {
        self.map
            .as_ref()
            .map(|m| m.run.party.len())
            .unwrap_or(0)
    }

    fn exit_to_reward(&mut self) -> Transition {
        let Some(map) = self.map.take() else {
            return Transition::Stay;
        };
        Transition::Goto(Screen::Reward(RewardScreen::new(
            map,
            self.gold,
            self.embers,
            self.items.clone(),
            self.kind,
        )))
    }
}

/// Catch rate for a fight kind. Returns None if uncatchable (final boss).
pub fn catch_rate(kind: NodeKind) -> Option<f32> {
    match kind {
        NodeKind::EasyFight => Some(0.70),
        NodeKind::NormalFight => Some(0.40),
        NodeKind::EliteFight => Some(0.20),
        // Currently the only Boss node is the run's final boss. Future
        // floor sub-bosses can switch this to Some(0.08).
        NodeKind::Boss => None,
        _ => None,
    }
}

pub fn has_net(items: &[ItemStack]) -> bool {
    items
        .iter()
        .any(|s| matches!(s.item, Item::MonsterNet) && s.count > 0)
}

fn consume_one_net(items: &mut Vec<ItemStack>) {
    if let Some(idx) = items
        .iter()
        .position(|s| matches!(s.item, Item::MonsterNet) && s.count > 0)
    {
        items[idx].count = items[idx].count.saturating_sub(1);
        if items[idx].count == 0 {
            items.remove(idx);
        }
    }
}

/// Build a PartyMember for a capture by looking up the species in the
/// starters registry first (in case it's a starter-class species), then
/// falling back to the bestiary. Returns None if the species isn't
/// known. Until Phase 5 unifies templates, this stuffs the species into
/// a synthesized Starter so the run can carry it.
fn build_party_member_from_capture(instance: &crate::meta::MonsterInstance) -> Option<PartyMember> {
    use crate::data::{enemies, starters};
    if let Some(starter) = starters::all_starters()
        .into_iter()
        .find(|s| s.name == instance.species)
    {
        return Some(PartyMember::from_starter(instance.id.clone(), starter));
    }
    if let Some(enemy) = enemies::all_enemies()
        .into_iter()
        .find(|e| e.name == instance.species)
    {
        // Synthesise a Starter wrapper from enemy data. Visual is a
        // single-frame fallback using the enemy sprite. Starting attacks
        // come from the enemy's moveset.
        use crate::data::starters::{Starter, StarterVisual};
        let visual = StarterVisual::Frames(vec![enemy.sprite.clone()]);
        let starter = Starter {
            name: enemy.name.clone(),
            primary_type: enemy.primary_type,
            starting_attacks: enemy.moveset.clone(),
            visual,
            palette: enemy.palette.clone(),
            description: enemy.description.clone(),
        };
        return Some(PartyMember::from_starter(instance.id.clone(), starter));
    }
    None
}
