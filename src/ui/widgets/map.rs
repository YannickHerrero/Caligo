use std::collections::HashSet;

use crate::map::{generate::FLOORS, MapGraph, MapNode, NodeId};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::{border, Marker},
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
const LABEL_COLOR: Color = Color::Rgb(225, 220, 210);

pub const CARD_WIDTH: u16 = 9;
pub const CARD_HEIGHT: u16 = 4;
pub const FLOOR_ROWS: u16 = 7;

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
    if area.width < CARD_WIDTH || area.height == 0 {
        return;
    }
    let reachable = reachable_set(graph);
    for node in &graph.nodes {
        let Some(card_rect) = card_rect(node, area, scroll) else {
            continue;
        };
        let state = node_state(node, graph, &reachable);
        let is_cursor = Some(node.id) == cursor;
        render_card(frame, node, state, is_cursor, pulse, card_rect);
    }
}

fn render_card(
    frame: &mut Frame,
    node: &MapNode,
    state: NodeState,
    is_cursor: bool,
    pulse: f32,
    card_rect: Rect,
) {
    let (border_color, label_color, icon_color) = card_colors(node, state, pulse);

    let mut border_style = Style::default().fg(border_color);
    if state == NodeState::Reachable || state == NodeState::Current {
        border_style = border_style.add_modifier(Modifier::BOLD);
    }
    if is_cursor {
        border_style = border_style.add_modifier(Modifier::BOLD);
    }

    let border_set = if is_cursor {
        border::DOUBLE
    } else if state == NodeState::Current {
        border::ROUNDED
    } else {
        border::PLAIN
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style);

    let inner = block.inner(card_rect);
    frame.render_widget(block, card_rect);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let icon_style = Style::default()
        .fg(icon_color)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(label_color);

    let lines = vec![
        Line::from(Span::styled(node.kind.icon(), icon_style)).alignment(Alignment::Center),
        Line::from(Span::styled(node.kind.card_label(), label_style))
            .alignment(Alignment::Center),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn card_colors(node: &MapNode, state: NodeState, pulse: f32) -> (Color, Color, Color) {
    match state {
        NodeState::Visited => (VISITED_COLOR, VISITED_COLOR, VISITED_COLOR),
        NodeState::Current => {
            let f = 0.75 + 0.25 * pulse;
            let c = dim_color(CURRENT_COLOR, f);
            (c, c, c)
        }
        NodeState::Reachable => {
            let f = 0.7 + 0.3 * pulse;
            let kind_color = dim_color(node.kind.color(), f);
            (kind_color, LABEL_COLOR, kind_color)
        }
        NodeState::Future => {
            let dim = dim_color(node.kind.color(), 0.45);
            let label = dim_color(LABEL_COLOR, 0.4);
            (dim, label, dim)
        }
    }
}

fn card_rect(node: &MapNode, area: Rect, scroll: i32) -> Option<Rect> {
    let (cx, csy) = node_unclipped_position(node, area, scroll);
    let half_w = CARD_WIDTH as i32 / 2;
    let half_h = CARD_HEIGHT as i32 / 2;
    let top_x = cx as i32 - half_w;
    let top_y = csy - half_h;
    let bottom_y = top_y + CARD_HEIGHT as i32;
    if bottom_y <= 0 || top_y >= area.height as i32 {
        return None;
    }
    if top_x < area.x as i32 || top_x + CARD_WIDTH as i32 > (area.x + area.width) as i32 {
        return None;
    }
    if top_y < 0 || bottom_y > area.height as i32 {
        // Skip cards that would clip vertically — keep cards intact for readability.
        return None;
    }
    Some(Rect {
        x: top_x as u16,
        y: area.y + top_y as u16,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
    })
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
    let half_h = CARD_HEIGHT as i32 / 2;
    for node in &graph.nodes {
        let (x1, sy1_center) = node_unclipped_position(node, area, scroll);
        // Edge starts at the top of the lower-floor card (smaller screen y).
        let sy1 = sy1_center - half_h;
        for &child_id in &node.children {
            let child = graph.node(child_id);
            let (x2, sy2_center) = node_unclipped_position(child, area, scroll);
            // Edge ends at the bottom of the higher-floor card.
            let sy2 = sy2_center + half_h;

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

fn node_unclipped_position(node: &MapNode, area: Rect, scroll: i32) -> (u16, i32) {
    let columns = crate::map::generate::COLUMNS as u16;
    let usable_w = area.width;
    let col_step = if columns <= 1 {
        0.0
    } else {
        (usable_w.saturating_sub(CARD_WIDTH)) as f32 / (columns - 1) as f32
    };
    let card_left = area.x as f32 + col_step * node.column as f32;
    let x = (card_left + CARD_WIDTH as f32 / 2.0) as u16;
    let virtual_y = floor_virtual_y(node.floor as i32);
    let sy = virtual_y - scroll;
    (x, sy)
}
