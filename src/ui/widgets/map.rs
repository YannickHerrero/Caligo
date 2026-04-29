use crate::map::{MapGraph, MapNode};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

const NODE_GLYPH: &str = "●";

pub fn render_nodes(frame: &mut Frame, graph: &MapGraph, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    for node in &graph.nodes {
        let Some((x, y)) = node_position(node, graph, area) else {
            continue;
        };
        let cell = Rect {
            x,
            y,
            width: 1,
            height: 1,
        };
        let span = Span::styled(NODE_GLYPH, Style::default().fg(Color::Gray));
        frame.render_widget(Paragraph::new(span), cell);
    }
}

pub fn node_position(node: &MapNode, graph: &MapGraph, area: Rect) -> Option<(u16, u16)> {
    let floors = graph.floor_count() as u16;
    if floors == 0 || area.width == 0 || area.height == 0 {
        return None;
    }
    let columns = crate::map::generate::COLUMNS as u16;
    let col_step = (area.width.saturating_sub(2)) as f32 / columns as f32;
    let row_step = (area.height.saturating_sub(2)) as f32 / floors as f32;

    // Boss at top, floor 0 at bottom: invert y.
    let inv_floor = (floors - 1) - node.floor as u16;
    let x = area.x + 1 + (col_step * (node.column as f32 + 0.5)) as u16;
    let y = area.y + 1 + (row_step * (inv_floor as f32 + 0.5)) as u16;
    Some((x, y))
}
