use std::collections::HashSet;

use crate::map::{MapGraph, MapNode, NodeId};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CanvasLine},
        Block, Borders, Paragraph,
    },
    Frame,
};

const EDGE_DIM: Color = Color::Rgb(60, 52, 44);
const EDGE_BRIGHT: Color = Color::Rgb(200, 170, 110);
const VISITED_COLOR: Color = Color::Rgb(90, 90, 96);
const CURRENT_COLOR: Color = Color::Rgb(255, 230, 170);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeState {
    Visited,
    Current,
    Reachable,
    Future,
}

pub fn render_header(frame: &mut Frame, graph: &MapGraph, area: Rect) {
    let total = graph.floor_count() as i32;
    let position_label: String = match graph.current {
        Some(id) => format!("Floor {} / {}", graph.node(id).floor as i32 + 1, total),
        None => format!("Floor 0 / {}  (start)", total),
    };
    let line = Line::from(vec![
        Span::styled(
            "Caligo — Map",
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ·   ", Style::default().fg(Color::DarkGray)),
        Span::styled(position_label, Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

pub fn render_info_panel(
    frame: &mut Frame,
    graph: &MapGraph,
    cursor: Option<NodeId>,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    if let Some(id) = cursor {
        let node = graph.node(id);
        lines.push(Line::from(vec![
            Span::styled(
                node.kind.icon(),
                Style::default()
                    .fg(node.kind.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                node.kind.label(),
                Style::default()
                    .fg(node.kind.color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            node.kind.description(),
            Style::default().fg(Color::Gray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "No reachable nodes.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(""));
    lines.push(controls_line());

    frame.render_widget(Paragraph::new(lines), inner);
}

fn controls_line() -> Line<'static> {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    Line::from(vec![
        Span::styled("← →", key),
        Span::styled(" navigate   ", dim),
        Span::styled("Enter", key),
        Span::styled(" select   ", dim),
        Span::styled("q", key),
        Span::styled(" back", dim),
    ])
}

pub fn render_edges(frame: &mut Frame, graph: &MapGraph, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let reachable = reachable_set(graph);
    let edges = collect_edges(graph, area, &reachable);
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
                    color: edge.color,
                });
            }
        });

    frame.render_widget(canvas, area);
}

pub fn render_nodes(
    frame: &mut Frame,
    graph: &MapGraph,
    cursor: Option<NodeId>,
    pulse: f32,
    area: Rect,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let reachable = reachable_set(graph);
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
        let state = node_state(node, graph, &reachable);
        let mut style = node_style(node, state);
        if state == NodeState::Reachable {
            let factor = 0.7 + 0.3 * pulse;
            style = style.fg(dim_color(node.kind.color(), factor));
        }
        if state == NodeState::Current {
            let factor = 0.75 + 0.25 * pulse;
            style = style.fg(dim_color(CURRENT_COLOR, factor));
        }
        if Some(node.id) == cursor {
            style = style.add_modifier(Modifier::REVERSED);
            if pulse > 0.5 {
                style = style.add_modifier(Modifier::BOLD);
            }
        }
        let span = Span::styled(node.kind.icon(), style);
        frame.render_widget(Paragraph::new(span), cell);
    }
}

fn reachable_set(graph: &MapGraph) -> HashSet<NodeId> {
    graph.reachable().into_iter().collect()
}

fn node_state(node: &MapNode, graph: &MapGraph, reachable: &HashSet<NodeId>) -> NodeState {
    if Some(node.id) == graph.current {
        NodeState::Current
    } else if node.visited {
        NodeState::Visited
    } else if reachable.contains(&node.id) {
        NodeState::Reachable
    } else {
        NodeState::Future
    }
}

fn node_style(node: &MapNode, state: NodeState) -> Style {
    match state {
        NodeState::Visited => Style::default().fg(VISITED_COLOR),
        NodeState::Current => Style::default()
            .fg(CURRENT_COLOR)
            .add_modifier(Modifier::BOLD),
        NodeState::Reachable => Style::default()
            .fg(node.kind.color())
            .add_modifier(Modifier::BOLD),
        NodeState::Future => Style::default().fg(dim_color(node.kind.color(), 0.45)),
    }
}

fn dim_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        other => other,
    }
}

#[derive(Clone, Copy)]
struct Edge {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: Color,
}

fn collect_edges(graph: &MapGraph, area: Rect, reachable: &HashSet<NodeId>) -> Vec<Edge> {
    let current = graph.current;
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
            let color = if Some(node.id) == current && reachable.contains(&child_id) {
                EDGE_BRIGHT
            } else if current.is_none() && reachable.contains(&child_id) {
                EDGE_BRIGHT
            } else {
                EDGE_DIM
            };
            edges.push(Edge {
                x1: (x1 - area.x) as f64 + 0.5,
                y1: (area.height as f64) - ((y1 - area.y) as f64 + 0.5),
                x2: (x2 - area.x) as f64 + 0.5,
                y2: (area.height as f64) - ((y2 - area.y) as f64 + 0.5),
                color,
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
