use crate::fight::FightState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_top_bar(frame: &mut Frame, fight: &FightState, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            format!("Floor {}", fight.floor),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  —  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            fight.enemy.name.clone(),
            Style::default()
                .fg(fight.enemy.color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let widget = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

const HP_BAR_WIDTH: usize = 16;

fn hp_bar(label: &str, hp: u32, max_hp: u32, color: Color) -> Line<'static> {
    let ratio = if max_hp == 0 {
        0.0
    } else {
        hp as f32 / max_hp as f32
    };
    let filled = (ratio * HP_BAR_WIDTH as f32).round() as usize;
    let filled = filled.min(HP_BAR_WIDTH);
    let empty = HP_BAR_WIDTH - filled;

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
            format!("{}/{}", hp, max_hp),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

pub fn render_hp_bars(frame: &mut Frame, fight: &FightState, area: Rect) {
    let player_bar = hp_bar(
        "Crab",
        fight.player_hp,
        fight.player_max_hp,
        Color::Rgb(255, 120, 80),
    );
    let player_width = 32u16.min(area.width);
    let player_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: player_width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(player_bar), player_area);

    let enemy_bar = hp_bar(
        &fight.enemy.name,
        fight.enemy.hp,
        fight.enemy.max_hp,
        fight.enemy.color,
    );
    let enemy_width = 32u16.min(area.width);
    let enemy_area = Rect {
        x: area.x + area.width.saturating_sub(enemy_width).saturating_sub(1),
        y: area.y,
        width: enemy_width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(enemy_bar), enemy_area);
}

pub fn render_action_menu(frame: &mut Frame, fight: &FightState, area: Rect) {
    use ratatui::widgets::{Block, Borders};
    use crate::fight::Action;

    let lines: Vec<Line> = Action::ALL
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let is_selected = i == fight.selected_action;
            let cursor = if is_selected { "> " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(vec![
                Span::styled(cursor, style),
                Span::styled(action.label(), style),
            ])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Actions ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let widget = Paragraph::new(lines).block(block);
    frame.render_widget(widget, area);
}

pub fn render_attack_menu(frame: &mut Frame, fight: &FightState, area: Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::widgets::{Block, Borders};
    use crate::fight::MAX_ATTACKS;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Attacks ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(inner);

    for row_idx in 0..2 {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(rows[row_idx]);

        for col_idx in 0..2 {
            let slot = row_idx * 2 + col_idx;
            let label = if slot < fight.attacks.len() {
                fight.attacks[slot].name.clone()
            } else if slot < MAX_ATTACKS {
                "—".to_string()
            } else {
                String::new()
            };

            let is_selected = slot == fight.attack_selected;
            let cursor = if is_selected { "> " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if slot < fight.attacks.len() {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::styled(cursor, style),
                Span::styled(label, style),
            ]);
            frame.render_widget(Paragraph::new(line), cols[col_idx]);
        }
    }
}

pub fn render_item_menu(frame: &mut Frame, fight: &FightState, area: Rect) {
    use ratatui::widgets::{Block, Borders};

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Items ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if fight.items.is_empty() || inner.height == 0 {
        return;
    }

    let visible = inner.height as usize;
    let scroll = fight.item_scroll.min(fight.items.len().saturating_sub(1));
    let end = (scroll + visible).min(fight.items.len());

    let has_more_above = scroll > 0;
    let has_more_below = end < fight.items.len();

    let mut lines: Vec<Line> = Vec::with_capacity(visible);
    for (idx, stack) in fight.items[scroll..end].iter().enumerate() {
        let global_idx = scroll + idx;
        let is_selected = global_idx == fight.item_selected;
        let cursor = if is_selected { "> " } else { "  " };
        let style = if is_selected {
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
        let mut spans = vec![
            Span::styled(cursor, style),
            Span::styled(label, style),
        ];

        if idx == 0 && has_more_above {
            spans.push(Span::styled(
                "  ↑",
                Style::default().fg(Color::DarkGray),
            ));
        } else if idx == visible.saturating_sub(1) && has_more_below {
            spans.push(Span::styled(
                "  ↓",
                Style::default().fg(Color::DarkGray),
            ));
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}
