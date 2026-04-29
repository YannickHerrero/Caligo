use crate::map::{MapGraph, MapNode};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{
        canvas::{Canvas, Line as CanvasLine},
        Paragraph,
    },
    Frame,
};

const NODE_GLYPH: &str = "●";
const EDGE_COLOR: Color = Color::Rgb(80, 70, 60);

pub fn render_edges(frame: &mut Frame, graph: &MapGraph, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let edges = collect_edges(graph, area);
    if edges.is_empty() {
        return;
    }

    let x_bounds = [0.0_f64, area.width as f64];
    let y_bounds = [0.0_f64, area.height as f64];

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .x_bounds(x_bounds)
        .y_bounds(y_bounds)
        .paint(move |ctx| {
            for edge in &edges {
                ctx.draw(&CanvasLine {
                    x1: edge.x1,
                    y1: edge.y1,
                    x2: edge.x2,
                    y2: edge.y2,
                    color: EDGE_COLOR,
                });
            }
        });

    frame.render_widget(canvas, area);
}

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

#[derive(Clone, Copy)]
struct Edge {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

fn collect_edges(graph: &MapGraph, area: Rect) -> Vec<Edge> {
    let mut edges = Vec::new();
    for node in &graph.nodes {
        let Some((x1, y1)) = node_position(node, graph, area) else {
            continue;
        };
        for &child_id in &node.children {
            let child = graph.node(child_id);
            let Some((x2, y2)) = node_position(child, graph, area) else {
                continue;
            };
            edges.push(Edge {
                x1: (x1 - area.x) as f64 + 0.5,
                y1: (area.height as f64) - ((y1 - area.y) as f64 + 0.5),
                x2: (x2 - area.x) as f64 + 0.5,
                y2: (area.height as f64) - ((y2 - area.y) as f64 + 0.5),
            });
        }
    }
    edges
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
