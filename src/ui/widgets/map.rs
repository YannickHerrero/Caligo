use std::collections::HashSet;

use crate::map::{generate::FLOORS, MapGraph, MapNode, NodeId};
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

pub const FLOOR_ROWS: u16 = 5;

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

pub fn compute_scroll(graph: &MapGraph, cursor: Option<NodeId>, viewport_height: u16) -> i32 {
    let focus_floor = cursor
        .or(graph.current)
        .map(|id| graph.node(id).floor as i32)
        .unwrap_or(0);
    let virtual_y = floor_virtual_y(focus_floor);
    let viewport_center = viewport_height as i32 / 2;
    let scroll = virtual_y - viewport_center;
    let max_scroll = (virtual_map_height() - viewport_height as i32).max(0);
    scroll.clamp(0, max_scroll)
}

pub fn virtual_map_height() -> i32 {
    FLOORS as i32 * FLOOR_ROWS as i32
}

fn floor_virtual_y(floor: i32) -> i32 {
    let inv = FLOORS as i32 - 1 - floor;
    inv * FLOOR_ROWS as i32 + FLOOR_ROWS as i32 / 2
}

pub fn render_edges(frame: &mut Frame, graph: &MapGraph, scroll: i32, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let reachable = reachable_set(graph);
    let edges = collect_edges(graph, area, scroll, &reachable);
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
    scroll: i32,
    area: Rect,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let reachable = reachable_set(graph);
    for node in &graph.nodes {
        let Some((x, y)) = node_screen_position(node, area, scroll) else {
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

fn collect_edges(
    graph: &MapGraph,
    area: Rect,
    scroll: i32,
    reachable: &HashSet<NodeId>,
) -> Vec<Edge> {
    let current = graph.current;
    let mut edges = Vec::new();
    for node in &graph.nodes {
        let (x1, sy1) = node_unclipped_position(node, area, scroll);
        for &child_id in &node.children {
            let child = graph.node(child_id);
            let (x2, sy2) = node_unclipped_position(child, area, scroll);
            // Skip when both endpoints are far outside viewport.
            let h = area.height as i32;
            let both_above = sy1 < 0 && sy2 < 0;
            let both_below = sy1 >= h && sy2 >= h;
            if both_above || both_below {
                continue;
            }

            let color = if Some(node.id) == current && reachable.contains(&child_id) {
                EDGE_BRIGHT
            } else if current.is_none() && reachable.contains(&child_id) {
                EDGE_BRIGHT
            } else {
                EDGE_DIM
            };
            let to_canvas_y = |sy: i32| (area.height as f64) - (sy as f64 + 0.5);
            edges.push(Edge {
                x1: (x1 - area.x) as f64 + 0.5,
                y1: to_canvas_y(sy1),
                x2: (x2 - area.x) as f64 + 0.5,
                y2: to_canvas_y(sy2),
                color,
            });
        }
    }
    edges
}

fn node_screen_position(node: &MapNode, area: Rect, scroll: i32) -> Option<(u16, u16)> {
    let (x, sy) = node_unclipped_position(node, area, scroll);
    if sy < 0 || sy >= area.height as i32 {
        return None;
    }
    Some((x, area.y + sy as u16))
}

fn node_unclipped_position(node: &MapNode, area: Rect, scroll: i32) -> (u16, i32) {
    let columns = crate::map::generate::COLUMNS as u16;
    let usable_w = area.width.saturating_sub(2);
    let col_step = if columns == 0 {
        0.0
    } else {
        usable_w as f32 / columns as f32
    };
    let x = area.x + 1 + (col_step * (node.column as f32 + 0.5)) as u16;
    let virtual_y = floor_virtual_y(node.floor as i32);
    let sy = virtual_y - scroll;
    (x, sy)
}
