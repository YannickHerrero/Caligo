use crate::crab::Crab;
use crate::fight::{Attack, Item, ItemStack};
use crate::player::Player;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const BAR_WIDTH: usize = 16;

pub fn render_crab_panel(frame: &mut Frame, crab: &Crab, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Crab ",
            Style::default().fg(Color::Gray),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let frame_text = crab.get_frame();
    let lines: Vec<Line> = frame_text
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(crab.color()).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    let para = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(para, inner);
}

pub fn render_stats_panel(frame: &mut Frame, player: &Player, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Stats ",
            Style::default().fg(Color::Gray),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    lines.push(stat_bar("HP", player.hp, player.max_hp(), Color::Rgb(255, 120, 120)));
    lines.push(stat_bar(
        "MP",
        player.mana,
        player.max_mana(),
        Color::Rgb(120, 160, 255),
    ));
    lines.push(Line::from(vec![
        Span::styled(
            "Gold ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", player.gold),
            Style::default()
                .fg(Color::Rgb(240, 210, 110))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Trinkets",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    )));
    for (i, slot) in player.equipped_trinkets.iter().enumerate() {
        let mut spans = vec![Span::styled(
            format!(" {}: ", i + 1),
            Style::default().fg(Color::DarkGray),
        )];
        match slot {
            Some(t) => {
                spans.push(Span::styled(
                    t.name().to_string(),
                    Style::default().fg(Color::Rgb(220, 180, 255)),
                ));
                let hp_bonus = t.bonus_max_hp();
                let mana_bonus = t.bonus_max_mana();
                if hp_bonus > 0 {
                    spans.push(Span::styled(
                        format!("  +{} HP", hp_bonus),
                        Style::default().fg(Color::Rgb(255, 120, 120)),
                    ));
                }
                if mana_bonus > 0 {
                    spans.push(Span::styled(
                        format!("  +{} MP", mana_bonus),
                        Style::default().fg(Color::Rgb(120, 160, 255)),
                    ));
                }
            }
            None => spans.push(Span::styled("—", Style::default().fg(Color::DarkGray))),
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn stat_bar(label: &str, current: u32, max: u32, color: Color) -> Line<'static> {
    let ratio = if max == 0 {
        0.0
    } else {
        current as f32 / max as f32
    };
    let filled = (ratio * BAR_WIDTH as f32).round() as usize;
    let filled = filled.min(BAR_WIDTH);
    let empty = BAR_WIDTH - filled;
    Line::from(vec![
        Span::styled(
            format!("{} ", label),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        Span::styled("] ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}/{}", current, max),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

pub fn render_attacks_panel(
    frame: &mut Frame,
    player: &Player,
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
            " Attacks ",
            Style::default().fg(if focused { Color::Yellow } else { Color::Gray }),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || player.owned_attacks.is_empty() {
        return;
    }

    let visible = inner.height as usize;
    let scroll = scroll.min(player.owned_attacks.len().saturating_sub(1));
    let end = (scroll + visible).min(player.owned_attacks.len());

    let has_more_above = scroll > 0;
    let has_more_below = end < player.owned_attacks.len();

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for (i, attack) in player.owned_attacks[scroll..end].iter().enumerate() {
        let global_idx = scroll + i;
        let is_selected = global_idx == cursor && focused;
        let cursor_str = if is_selected { "> " } else { "  " };

        let equipped_slot = player
            .equipped_attacks
            .iter()
            .position(|s| *s == Some(global_idx));
        let slot_marker = match equipped_slot {
            Some(s) => format!("[{}] ", s + 1),
            None => "    ".to_string(),
        };

        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if equipped_slot.is_some() {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let mut spans = vec![
            Span::styled(cursor_str, style),
            Span::styled(slot_marker, Style::default().fg(Color::DarkGray)),
            Span::styled(attack.name.clone(), style),
        ];
        if i == 0 && has_more_above {
            spans.push(Span::styled(
                "  ↑",
                Style::default().fg(Color::DarkGray),
            ));
        } else if i == visible.saturating_sub(1) && has_more_below {
            spans.push(Span::styled(
                "  ↓",
                Style::default().fg(Color::DarkGray),
            ));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_inventory_panel(
    frame: &mut Frame,
    player: &Player,
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
            Style::default().fg(if focused { Color::Yellow } else { Color::Gray }),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    if player.inventory.is_empty() {
        let line = Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(Paragraph::new(line), inner);
        return;
    }

    let visible = inner.height as usize;
    let scroll = scroll.min(player.inventory.len().saturating_sub(1));
    let end = (scroll + visible).min(player.inventory.len());
    let has_more_above = scroll > 0;
    let has_more_below = end < player.inventory.len();

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for (i, stack) in player.inventory[scroll..end].iter().enumerate() {
        let global_idx = scroll + i;
        let is_selected = global_idx == cursor && focused;
        let cursor_str = if is_selected { "> " } else { "  " };

        let trinket_equipped = is_trinket_equipped(stack, player);
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(stack.item.color())
        };

        let count_label = if stack.count > 1 {
            format!("  x{}", stack.count)
        } else {
            String::new()
        };
        let equipped_marker = if trinket_equipped { " ★" } else { "" };

        let mut spans = vec![
            Span::styled(cursor_str, style),
            Span::styled(stack.item.name(), style),
            Span::styled(count_label, Style::default().fg(Color::DarkGray)),
            Span::styled(
                equipped_marker,
                Style::default().fg(Color::Rgb(220, 180, 255)),
            ),
        ];
        if i == 0 && has_more_above {
            spans.push(Span::styled(
                "  ↑",
                Style::default().fg(Color::DarkGray),
            ));
        } else if i == visible.saturating_sub(1) && has_more_below {
            spans.push(Span::styled(
                "  ↓",
                Style::default().fg(Color::DarkGray),
            ));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn is_trinket_equipped(stack: &ItemStack, player: &Player) -> bool {
    if let Item::Trinket(kind) = &stack.item {
        player
            .equipped_trinkets
            .iter()
            .any(|slot| slot.as_ref() == Some(kind))
    } else {
        false
    }
}

pub fn render_action_message_strip(frame: &mut Frame, message: &str, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Action ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let line = Line::from(Span::styled(
        message.to_string(),
        Style::default().fg(Color::White),
    ));
    frame.render_widget(Paragraph::new(line), inner);
}

pub fn render_assign_strip(frame: &mut Frame, attack: &Attack, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " Equip ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let line = Line::from(vec![
        Span::styled(
            "Press 1–4 to equip ",
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            attack.name.clone(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  ·  Esc cancels",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), inner);
}

pub fn render_item_info_strip(frame: &mut Frame, item: &Item, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Info ",
            Style::default().fg(Color::Gray),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let line = Line::from(vec![
        Span::styled(
            item.name(),
            Style::default().fg(item.color()).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  —  ", Style::default().fg(Color::DarkGray)),
        Span::styled(item.description(), Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(Paragraph::new(line), inner);
}

pub fn render_info_strip(frame: &mut Frame, attack: Option<&Attack>, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Info ",
            Style::default().fg(Color::Gray),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let line = match attack {
        Some(a) => Line::from(vec![
            Span::styled(
                a.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(a.effect.label(), Style::default().fg(a.effect.color())),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("MP {}", a.mana_cost),
                Style::default().fg(Color::Rgb(120, 160, 255)),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                a.element.label().to_string(),
                Style::default().fg(a.element.color()),
            ),
            Span::styled("  —  ", Style::default().fg(Color::DarkGray)),
            Span::styled(a.description.clone(), Style::default().fg(Color::Gray)),
        ]),
        None => Line::from(Span::styled(
            "Tab to map · ←/→ switch panel · ↑/↓ scroll",
            Style::default().fg(Color::DarkGray),
        )),
    };
    frame.render_widget(Paragraph::new(line), inner);
}

