use crate::data::starters::{Starter, StarterVisual};
use crate::fight::{Item, ItemStack, PotionSize};
use crate::player::Player;
use crate::run::PartyMember;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::MapScreen;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InfoFocus {
    Party,
    Inventory,
}

/// In-run party menu, accessed from MapScreen via Tab. Lists every
/// party member with their HP/MP bars, shows a detail card for the
/// highlighted member (visual, stats, attacks), and lets the player
/// use inventory items on that member.
pub struct PlayerInfoScreen {
    pub map: Option<Box<MapScreen>>,
    focus: InfoFocus,
    party_cursor: usize,
    item_cursor: usize,
    item_scroll: usize,
    last_action_message: Option<String>,
    last_inventory_height: u16,
}

impl PlayerInfoScreen {
    pub fn new(map: MapScreen) -> Self {
        let party_cursor = map.run.active;
        Self {
            map: Some(Box::new(map)),
            focus: InfoFocus::Party,
            party_cursor,
            item_cursor: 0,
            item_scroll: 0,
            last_action_message: None,
            last_inventory_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Tab => return self.return_to_map(),
            KeyCode::Up | KeyCode::Char('k') => {
                self.last_action_message = None;
                self.scroll_focused(-1, player);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.last_action_message = None;
                self.scroll_focused(1, player);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.last_action_message = None;
                self.focus = InfoFocus::Party;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.last_action_message = None;
                self.focus = InfoFocus::Inventory;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if matches!(self.focus, InfoFocus::Inventory) {
                    self.use_focused_item(player);
                }
            }
            _ => {}
        }
        Transition::Stay
    }

    fn scroll_focused(&mut self, delta: i32, player: &Player) {
        match self.focus {
            InfoFocus::Party => {
                let len = self.party_len();
                if len > 0 {
                    let new_cursor = (self.party_cursor as i32 + delta)
                        .clamp(0, len as i32 - 1) as usize;
                    self.party_cursor = new_cursor;
                }
            }
            InfoFocus::Inventory => scroll_list(
                &mut self.item_cursor,
                &mut self.item_scroll,
                player.inventory.len(),
                self.last_inventory_height as usize,
                delta,
            ),
        }
    }

    fn party_len(&self) -> usize {
        self.map.as_ref().map(|m| m.run.party.len()).unwrap_or(0)
    }

    fn use_focused_item(&mut self, player: &mut Player) {
        if self.item_cursor >= player.inventory.len() {
            return;
        }
        let target = self.party_cursor;
        // Reach the active member's index up front so we can keep
        // Player in sync if we mutate the active slot directly.
        let Some(map) = self.map.as_mut() else {
            return;
        };
        let active = map.run.active;
        let Some(target_member) = map.run.party.get_mut(target) else {
            return;
        };

        let item = player.inventory[self.item_cursor].item.clone();
        let result = match &item {
            Item::HpPotion(size) => {
                let amount = match size {
                    PotionSize::Small => 10,
                    PotionSize::Large => 30,
                };
                let before = target_member.current_hp;
                target_member.current_hp =
                    (target_member.current_hp + amount).min(target_member.max_hp);
                let healed = target_member.current_hp.saturating_sub(before);
                consume_one(&mut player.inventory, self.item_cursor);
                if target == active {
                    player.hp = target_member.current_hp;
                }
                Some(format!(
                    "{} recovered {} HP.",
                    target_member.template.name, healed
                ))
            }
            Item::ManaPotion(size) => {
                let amount = match size {
                    PotionSize::Small => 6,
                    PotionSize::Large => 15,
                };
                let before = target_member.current_mana;
                target_member.current_mana =
                    (target_member.current_mana + amount).min(target_member.max_mana);
                let healed = target_member.current_mana.saturating_sub(before);
                consume_one(&mut player.inventory, self.item_cursor);
                if target == active {
                    player.mana = target_member.current_mana;
                }
                Some(format!(
                    "{} recovered {} MP.",
                    target_member.template.name, healed
                ))
            }
            Item::AttackStone { attack_name } => {
                if target_member
                    .attacks
                    .iter()
                    .any(|a| &a.name == attack_name)
                {
                    Some(format!("{} already knows that move.", target_member.template.name))
                } else if let Some(real) =
                    crate::data::attacks::find_by_name(attack_name)
                {
                    target_member.attacks.push(real);
                    consume_one(&mut player.inventory, self.item_cursor);
                    if target == active {
                        // Re-pull attacks into Player so the next fight
                        // sees the new move in the list.
                        let snapshot = target_member.clone();
                        player.sync_from_member(&snapshot);
                    }
                    Some(format!(
                        "{} learned {}!",
                        target_member.template.name, attack_name
                    ))
                } else {
                    Some("That stone is unreadable.".to_string())
                }
            }
            // Items that don't apply to a single member just no-op here
            // — Revive Pearl is combat-only, MonsterNet is post-fight,
            // GoldPouch is meta. Trinkets get a clearer UX once the per-
            // member equip flow exists.
            Item::Utility(_) | Item::Trinket(_) | Item::MonsterNet => {
                Some("That item can't be used from here.".to_string())
            }
        };

        self.last_action_message = result;
        if self.item_cursor >= player.inventory.len()
            && !player.inventory.is_empty()
        {
            self.item_cursor = player.inventory.len() - 1;
        }
    }

    fn return_to_map(&mut self) -> Transition {
        match self.map.take() {
            Some(map) => Transition::Goto(Screen::Map(*map)),
            None => Transition::Goto(Screen::Map(MapScreen::new())),
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, player: &Player) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Min(8),    // body
                Constraint::Length(1), // hint
                Constraint::Length(1), // message
            ])
            .split(area);

        render_title(frame, chunks[0]);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(body[0]);
        let party_list_area = left[0];
        let detail_area = left[1];
        let inventory_area = body[1];

        self.last_inventory_height = inventory_area.height.saturating_sub(2);

        let party = self.party_snapshot();
        let active = self.active_index();
        render_party_list(
            frame,
            &party,
            self.party_cursor,
            active,
            self.focus == InfoFocus::Party,
            party_list_area,
        );
        render_detail(frame, party.get(self.party_cursor), detail_area);
        render_inventory(
            frame,
            &player.inventory,
            self.item_cursor,
            self.item_scroll,
            self.focus == InfoFocus::Inventory,
            inventory_area,
        );

        render_hint(frame, self.focus, chunks[2]);
        if let Some(msg) = self.last_action_message.as_deref() {
            render_message(frame, msg, chunks[3]);
        }
    }

    fn party_snapshot(&self) -> Vec<PartyMember> {
        self.map
            .as_ref()
            .map(|m| m.run.party.clone())
            .unwrap_or_default()
    }

    fn active_index(&self) -> usize {
        self.map.as_ref().map(|m| m.run.active).unwrap_or(0)
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Caligo \u{2014} Party",
        Style::default()
            .fg(Color::Rgb(255, 140, 90))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_party_list(
    frame: &mut Frame,
    party: &[PartyMember],
    cursor: usize,
    active: usize,
    focused: bool,
    area: Rect,
) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " Team ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    if party.is_empty() {
        let line = Line::from(Span::styled(
            "(no monsters)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (idx, member) in party.iter().enumerate() {
        let is_selected = idx == cursor;
        let is_active = idx == active;
        let cursor_marker = if is_selected { "\u{25B6} " } else { "  " };
        let name_style = if is_selected && focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default()
                .fg(member.template.primary_type.color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let suffix = if is_active { "  *" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(cursor_marker, name_style),
            Span::styled(format!("{}{}", member.template.name, suffix), name_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!(
                    "HP {}/{}   MP {}/{}",
                    member.current_hp,
                    member.max_hp,
                    member.current_mana,
                    member.max_mana
                ),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_detail(frame: &mut Frame, member: Option<&PartyMember>, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Details ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let Some(member) = member else {
        let line = Line::from(Span::styled(
            "(no member)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // name + type
            Constraint::Length(5), // visual
            Constraint::Length(2), // stats
            Constraint::Min(1),    // attacks
        ])
        .split(inner);

    render_name(frame, &member.template, chunks[0]);
    render_visual(frame, &member.template, chunks[1]);
    render_stats(frame, member, chunks[2]);
    render_attacks(frame, member, chunks[3]);
}

fn render_name(frame: &mut Frame, template: &Starter, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            template.name.clone(),
            Style::default()
                .fg(template.primary_type.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   ({})", template.primary_type.label()),
            Style::default().fg(template.primary_type.color()),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_visual(frame: &mut Frame, template: &Starter, area: Rect) {
    let color = template.color();
    let lines: Vec<Line> = match &template.visual {
        StarterVisual::AnimatedCrab => {
            const FRAME: &[&str] = &[
                "    _~^~^~_     ",
                "\\) /  o   o  \\ (/",
                "  '_   ---   _'  ",
                "  \\ '-------' /  ",
            ];
            FRAME
                .iter()
                .map(|row| {
                    Line::from(Span::styled(
                        (*row).to_string(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ))
                })
                .collect()
        }
        StarterVisual::Frames(frames) => {
            if let Some(frame_lines) = frames.first() {
                frame_lines
                    .iter()
                    .map(|row| {
                        Line::from(Span::styled(
                            row.clone(),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ))
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
    };
    let h = lines.len() as u16;
    let pad = area.height.saturating_sub(h) / 2;
    let mut padded: Vec<Line> = Vec::with_capacity(lines.len() + pad as usize);
    for _ in 0..pad {
        padded.push(Line::from(""));
    }
    padded.extend(lines);
    frame.render_widget(
        Paragraph::new(padded).alignment(Alignment::Center),
        area,
    );
}

fn render_stats(frame: &mut Frame, member: &PartyMember, area: Rect) {
    let dim = Style::default().fg(Color::DarkGray);
    let line1 = Line::from(vec![
        Span::styled("HP ", dim),
        Span::styled(
            format!("{}/{}", member.current_hp, member.max_hp),
            Style::default().fg(Color::Rgb(255, 120, 80)).add_modifier(Modifier::BOLD),
        ),
        Span::styled("    MP ", dim),
        Span::styled(
            format!("{}/{}", member.current_mana, member.max_mana),
            Style::default().fg(Color::Rgb(120, 160, 255)).add_modifier(Modifier::BOLD),
        ),
    ]);
    let atk_pct = (member.attack_boost_pct * 100.0).round() as u32;
    let line2 = Line::from(vec![
        Span::styled("Speed ", dim),
        Span::styled(
            format!("{}", member.speed),
            Style::default().fg(Color::Rgb(180, 220, 200)).add_modifier(Modifier::BOLD),
        ),
        Span::styled("    Atk ", dim),
        Span::styled(
            format!("+{}%", atk_pct),
            Style::default().fg(Color::Rgb(255, 140, 90)).add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(vec![line1, line2]).alignment(Alignment::Center),
        area,
    );
}

fn render_attacks(frame: &mut Frame, member: &PartyMember, area: Rect) {
    if area.height == 0 {
        return;
    }
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Moves",
        Style::default().fg(Color::DarkGray),
    )));
    if member.attacks.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for attack in &member.attacks {
            let cost = if attack.mana_cost == 0 {
                String::new()
            } else {
                format!("  MP {}", attack.mana_cost)
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", attack.name),
                    Style::default()
                        .fg(attack.element.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(cost, Style::default().fg(Color::DarkGray)),
            ]));
        }
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_inventory(
    frame: &mut Frame,
    inventory: &[ItemStack],
    cursor: usize,
    scroll: usize,
    focused: bool,
    area: Rect,
) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " Inventory ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    if inventory.is_empty() {
        let line = Line::from(Span::styled(
            "(empty)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let visible = inner.height as usize;
    let scroll = scroll.min(inventory.len().saturating_sub(1));
    let end = (scroll + visible).min(inventory.len());

    let mut lines: Vec<Line> = Vec::new();
    for (idx, stack) in inventory[scroll..end].iter().enumerate() {
        let global_idx = scroll + idx;
        let is_selected = global_idx == cursor;
        let cursor_marker = if is_selected { "\u{25B6} " } else { "  " };
        let style = if is_selected && focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let label = if stack.count > 1 {
            format!("{}  x{}", stack.item.name(), stack.count)
        } else {
            stack.item.name()
        };
        lines.push(Line::from(vec![
            Span::styled(cursor_marker, style),
            Span::styled(label, style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_hint(frame: &mut Frame, focus: InfoFocus, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let mut spans = vec![
        Span::styled("\u{2191}\u{2193}", key),
        Span::styled(" navigate   ", dim),
        Span::styled("\u{2190}\u{2192}", key),
        Span::styled(" focus   ", dim),
    ];
    if matches!(focus, InfoFocus::Inventory) {
        spans.extend([
            Span::styled("Enter", key),
            Span::styled(" use on selected   ", dim),
        ]);
    }
    spans.extend([
        Span::styled("Tab", key),
        Span::styled(" / ", dim),
        Span::styled("Esc", key),
        Span::styled(" back to map", dim),
    ]);
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn render_message(frame: &mut Frame, message: &str, area: Rect) {
    let line = Line::from(Span::styled(
        message.to_string(),
        Style::default().fg(Color::Rgb(255, 210, 110)),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn consume_one(inventory: &mut Vec<ItemStack>, idx: usize) {
    if idx >= inventory.len() {
        return;
    }
    inventory[idx].count = inventory[idx].count.saturating_sub(1);
    if inventory[idx].count == 0 {
        inventory.remove(idx);
    }
}

fn scroll_list(cursor: &mut usize, scroll: &mut usize, len: usize, visible: usize, delta: i32) {
    if len == 0 {
        *cursor = 0;
        *scroll = 0;
        return;
    }
    let new_cursor = (*cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
    *cursor = new_cursor;
    if visible > 0 {
        if *cursor < *scroll {
            *scroll = *cursor;
        } else if *cursor >= *scroll + visible {
            *scroll = *cursor + 1 - visible;
        }
    }
}
