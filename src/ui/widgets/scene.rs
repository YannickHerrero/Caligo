use crate::crab::Crab;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

pub fn render_crab(frame: &mut Frame, crab: &Crab, area: Rect) {
    let crab_frame = crab.get_frame();
    let color = crab.color();

    let x_offset = crab.position.0 as u16;
    let y_offset = crab.position.1 as u16;

    let lines: Vec<Line> = crab_frame
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let crab_text = Text::from(lines);

    let crab_area = Rect {
        x: area.x + x_offset.min(area.width.saturating_sub(22)),
        y: area.y + y_offset.min(area.height.saturating_sub(5)),
        width: 22.min(area.width),
        height: 4.min(area.height),
    };

    let paragraph = Paragraph::new(crab_text);
    frame.render_widget(paragraph, crab_area);
}
