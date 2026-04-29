use crate::crab::Crab;
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
        let label = match slot {
            Some(t) => Span::styled(
                t.name().to_string(),
                Style::default().fg(Color::Rgb(220, 180, 255)),
            ),
            None => Span::styled("—", Style::default().fg(Color::DarkGray)),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", i + 1),
                Style::default().fg(Color::DarkGray),
            ),
            label,
        ]));
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
