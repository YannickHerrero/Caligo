use crate::data::attacks as attack_lib;
use crate::fight::{Attack, Effect, Item, ItemStack, PotionSize, TrinketKind, UtilityKind};
use crate::map::NodeKind;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::MapScreen;
use crossterm::event::KeyCode;
use rand::seq::SliceRandom;
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct RewardScreen {
    pub map: Option<Box<MapScreen>>,
    pub gold: u32,
    pub embers: u32,
    pub items: Vec<Item>,
    pub kind: NodeKind,
}

impl RewardScreen {
    pub fn new(
        map: Box<MapScreen>,
        gold: u32,
        embers: u32,
        items: Vec<Item>,
        kind: NodeKind,
    ) -> Self {
        Self {
            map: Some(map),
            gold,
            embers,
            items,
            kind,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        match key {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') | KeyCode::Esc => {
                match self.map.take() {
                    Some(map) => Transition::Goto(Screen::Map(*map)),
                    None => Transition::Stay,
                }
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

        let panel_w = 50.min(area.width.saturating_sub(4)).max(20);
        let panel_h = (5 + self.items.len() as u16 + 4).min(area.height.saturating_sub(2));
        let panel = Rect {
            x: area.x + (area.width.saturating_sub(panel_w)) / 2,
            y: area.y + (area.height.saturating_sub(panel_h)) / 2,
            width: panel_w,
            height: panel_h,
        };

        let title = match self.kind {
            NodeKind::Boss => " ★ Victory! ★ ",
            NodeKind::EliteFight => " Elite cleared! ",
            _ => " Victory! ",
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(255, 210, 110)))
            .title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Rgb(255, 210, 110))
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(panel);
        frame.render_widget(block, panel);
        if inner.height == 0 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // gold
                Constraint::Length(1), // embers
                Constraint::Min(1),    // items
                Constraint::Length(2), // hint
            ])
            .split(inner);

        let gold_line = Line::from(vec![
            Span::styled(
                format!("+{}", self.gold),
                Style::default()
                    .fg(Color::Rgb(240, 210, 110))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" gold", Style::default().fg(Color::Gray)),
        ]);
        frame.render_widget(
            Paragraph::new(gold_line).alignment(Alignment::Center),
            chunks[0],
        );

        if self.embers > 0 {
            let ember_line = Line::from(vec![
                Span::styled(
                    format!("+{}", self.embers),
                    Style::default()
                        .fg(Color::Rgb(255, 140, 90))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ember", Style::default().fg(Color::Gray)),
            ]);
            frame.render_widget(
                Paragraph::new(ember_line).alignment(Alignment::Center),
                chunks[1],
            );
        }

        let mut item_lines: Vec<Line> = Vec::new();
        if self.items.is_empty() {
            item_lines.push(Line::from(Span::styled(
                "(no item drops)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for item in &self.items {
                item_lines.push(Line::from(vec![
                    Span::styled(
                        "+ ",
                        Style::default().fg(Color::Rgb(180, 220, 130)),
                    ),
                    Span::styled(
                        item.name(),
                        Style::default().fg(item.color()).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }
        frame.render_widget(
            Paragraph::new(item_lines).alignment(Alignment::Center),
            chunks[2],
        );

        let hint = Line::from(vec![
            Span::styled(
                "Enter",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                " continue",
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(hint).alignment(Alignment::Center),
            chunks[3],
        );
    }
}

/// Roll a fight reward: gold amount and any item drops.
pub fn roll_rewards<R: Rng>(kind: NodeKind, player: &Player, rng: &mut R) -> (u32, Vec<Item>) {
    let gold = match kind {
        NodeKind::EasyFight => rng.gen_range(8..=15),
        NodeKind::NormalFight => rng.gen_range(15..=30),
        NodeKind::EliteFight => rng.gen_range(40..=60),
        NodeKind::Boss => rng.gen_range(80..=120),
        _ => 0,
    };
    let items = roll_drops(kind, player, rng);
    (gold, items)
}

fn roll_drops<R: Rng>(kind: NodeKind, player: &Player, rng: &mut R) -> Vec<Item> {
    let lucky = has_lucky_shell(player);
    // Lucky Shell doubles the per-fight chance that anything drops, capped
    // at 100%. Doesn't stack — only one shell counts even if two are
    // equipped. Guaranteed-drop tiers (Elite, Boss) are unaffected.
    let scale = |p: f64| -> f64 {
        if lucky { (p * 2.0).min(1.0) } else { p }
    };
    let mut out = match kind {
        NodeKind::EasyFight => {
            if !rng.gen_bool(scale(0.25)) {
                Vec::new()
            } else if rng.gen_bool(0.5) {
                vec![Item::HpPotion(PotionSize::Small)]
            } else {
                stone_for_stage(kind, rng).into_iter().collect()
            }
        }
        NodeKind::NormalFight => {
            if !rng.gen_bool(scale(0.5)) {
                Vec::new()
            } else {
                let roll = rng.gen_range(0..100);
                if roll < 30 {
                    vec![Item::HpPotion(PotionSize::Small)]
                } else if roll < 60 {
                    vec![Item::ManaPotion(PotionSize::Small)]
                } else {
                    stone_for_stage(kind, rng).into_iter().collect()
                }
            }
        }
        NodeKind::EliteFight => {
            // Always one item from the elite pool.
            let roll = rng.gen_range(0..100);
            if roll < 25 {
                vec![Item::HpPotion(PotionSize::Large)]
            } else if roll < 50 {
                vec![Item::ManaPotion(PotionSize::Large)]
            } else if roll < 65 {
                vec![Item::Utility(UtilityKind::Revive)]
            } else if roll < 70 {
                vec![Item::Trinket(random_trinket(rng))]
            } else {
                stone_for_stage(kind, rng)
                    .map(|s| vec![s])
                    .unwrap_or_default()
            }
        }
        NodeKind::Boss => {
            // Boss reward: a trinket, a large HP potion, a stone, and a
            // guaranteed Monster Net (rewards capture attempts on the
            // next run).
            let mut out = vec![
                Item::Trinket(random_trinket(rng)),
                Item::HpPotion(PotionSize::Large),
                Item::MonsterNet,
            ];
            if let Some(stone) = stone_for_stage(kind, rng) {
                out.push(stone);
            }
            out
        }
        _ => Vec::new(),
    };
    // Independent small chance per non-boss fight to drop a Monster Net.
    // Lucky Shell doubles the base chance like other drops.
    let net_base = match kind {
        NodeKind::EasyFight => 0.10,
        NodeKind::NormalFight => 0.15,
        NodeKind::EliteFight => 0.25,
        _ => 0.0,
    };
    if net_base > 0.0 && rng.gen_bool(scale(net_base)) {
        out.push(Item::MonsterNet);
    }
    out
}

/// How powerful an attack is, used to bucket its stone into a drop tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoneTier {
    Easy,
    Normal,
    Elite,
    Boss,
}

fn attack_tier(attack: &Attack) -> StoneTier {
    match attack.effect {
        Effect::Damage(n) => match n {
            0..=7 => StoneTier::Easy,
            8..=12 => StoneTier::Normal,
            13..=18 => StoneTier::Elite,
            _ => StoneTier::Boss,
        },
        Effect::Heal(n) => match n {
            0..=10 => StoneTier::Easy,
            11..=18 => StoneTier::Normal,
            _ => StoneTier::Elite,
        },
        // Buffs are utility — keep them in the easy bucket so they're a
        // common-but-flexible find rather than a high-tier reward.
        Effect::Buff { .. } => StoneTier::Easy,
    }
}

/// Per-stage weights for picking which tier of stone drops. Each entry is
/// [easy, normal, elite, boss]. Lower stages are heavily weighted toward
/// the easy bucket but always leave a sliver of chance for higher tiers.
fn stage_tier_weights(stage: NodeKind) -> [u32; 4] {
    match stage {
        NodeKind::EasyFight => [70, 20, 8, 2],
        NodeKind::NormalFight => [25, 55, 15, 5],
        NodeKind::EliteFight => [10, 30, 50, 10],
        NodeKind::Boss => [5, 15, 40, 40],
        _ => [100, 0, 0, 0],
    }
}

pub fn stone_for_stage<R: Rng>(stage: NodeKind, rng: &mut R) -> Option<Item> {
    let weights = stage_tier_weights(stage);
    let tier = pick_tier(rng, weights);
    let attacks = attack_lib::all_attacks();
    let candidates: Vec<Attack> = attacks
        .into_iter()
        .filter(|a| attack_tier(a) == tier)
        .collect();
    let chosen = candidates.choose(rng)?;
    Some(Item::AttackStone {
        attack_name: chosen.name.clone(),
    })
}

fn pick_tier<R: Rng>(rng: &mut R, weights: [u32; 4]) -> StoneTier {
    let total: u32 = weights.iter().sum();
    if total == 0 {
        return StoneTier::Easy;
    }
    let mut r = rng.gen_range(0..total);
    let tiers = [
        StoneTier::Easy,
        StoneTier::Normal,
        StoneTier::Elite,
        StoneTier::Boss,
    ];
    for (i, w) in weights.iter().enumerate() {
        if r < *w {
            return tiers[i];
        }
        r -= w;
    }
    StoneTier::Easy
}

fn has_lucky_shell(player: &Player) -> bool {
    player
        .equipped_trinkets
        .iter()
        .any(|slot| matches!(slot, Some(TrinketKind::LuckyShell)))
}

fn random_trinket<R: Rng>(rng: &mut R) -> TrinketKind {
    match rng.gen_range(0..3) {
        0 => TrinketKind::HeartCharm,
        1 => TrinketKind::ManaPearl,
        _ => TrinketKind::LuckyShell,
    }
}

/// Apply a reward to the player: add gold and stack the items into the
/// inventory.
pub fn apply_rewards(player: &mut Player, gold: u32, items: &[Item]) {
    player.gold = player.gold.saturating_add(gold);
    for item in items {
        if let Some(stack) = player
            .inventory
            .iter_mut()
            .find(|s| same_item(&s.item, item))
        {
            stack.count = stack.count.saturating_add(1);
        } else {
            player.inventory.push(ItemStack::new(item.clone(), 1));
        }
    }
}

fn same_item(a: &Item, b: &Item) -> bool {
    match (a, b) {
        (Item::HpPotion(s1), Item::HpPotion(s2)) => s1 == s2,
        (Item::ManaPotion(s1), Item::ManaPotion(s2)) => s1 == s2,
        (Item::AttackStone { attack_name: n1 }, Item::AttackStone { attack_name: n2 }) => n1 == n2,
        (Item::Trinket(k1), Item::Trinket(k2)) => k1 == k2,
        (Item::Utility(k1), Item::Utility(k2)) => k1 == k2,
        _ => false,
    }
}
