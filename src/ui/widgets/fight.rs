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
        y: area.y + area.height.saturating_sub(2),
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
