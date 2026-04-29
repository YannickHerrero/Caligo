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

pub fn render_confirm_popup(frame: &mut Frame, node: &MapNode, area: Rect) {
    let kind = node.kind;
    let popup_w: u16 = 44.min(area.width.saturating_sub(4)).max(20);
    let popup_h: u16 = 9.min(area.height.saturating_sub(2)).max(5);
    if popup_w < 20 || popup_h < 5 {
        return;
    }
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    // Dim the rest of the screen so the popup feels modal.
    let buf = frame.buffer_mut();
    for y in area.y..(area.y + area.height) {
        for x in area.x..(area.x + area.width) {
            if x >= popup.x && x < popup.x + popup.width && y >= popup.y && y < popup.y + popup.height {
                continue;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                let fg = match cell.fg {
                    Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                    other => other,
                };
                cell.set_fg(fg);
            }
        }
    }

    // Clear popup background to opaque
    for y in popup.y..(popup.y + popup.height) {
        for x in popup.x..(popup.x + popup.width) {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_style(Style::default());
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(
            Style::default()
                .fg(kind.color())
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    if inner.height == 0 {
        return;
    }

    let kind_style = Style::default()
        .fg(kind.color())
        .add_modifier(Modifier::BOLD);
    let label_line = Line::from(vec![
        Span::styled(kind.icon(), kind_style),
        Span::raw("  "),
        Span::styled(kind.label(), kind_style),
    ])
    .alignment(Alignment::Center);

    let desc_line = Line::from(Span::styled(
        kind.description(),
        Style::default().fg(Color::Gray),
    ))
    .alignment(Alignment::Center);

    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let prompt = Line::from(vec![
        Span::styled("Enter", key),
        Span::styled(" to enter   ", dim),
        Span::styled("Esc", key),
        Span::styled(" cancel", dim),
    ])
    .alignment(Alignment::Center);

    let lines = vec![
        Line::from(""),
        label_line,
        Line::from(""),
        desc_line,
        Line::from(""),
        prompt,
    ];
    frame.render_widget(Paragraph::new(lines), inner);
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
        Span::styled(" cursor   ", dim),
        Span::styled("↑ ↓", key),
        Span::styled(" scroll   ", dim),
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
        let (top_x, top_y) = card_top_left(node, area, scroll);
        if !card_overlaps_area(top_x, top_y, area) {
            continue;
        }
        let state = node_state(node, graph, &reachable);
        let is_cursor = Some(node.id) == cursor;
        render_card(frame, node, state, is_cursor, pulse, top_x, top_y, area);
    }
}

fn render_card(
    frame: &mut Frame,
    node: &MapNode,
    state: NodeState,
    is_cursor: bool,
    pulse: f32,
    top_x: i32,
    top_y: i32,
    area: Rect,
) {
    let (border_color, label_color, icon_color) = card_colors(node, state, pulse);

    let mut border_style = Style::default().fg(border_color);
    if state == NodeState::Reachable || state == NodeState::Current || is_cursor {
        border_style = border_style.add_modifier(Modifier::BOLD);
    }
    let icon_style = Style::default()
        .fg(icon_color)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(label_color);
    let inside_style = Style::default();

    let bs = if is_cursor {
        border::DOUBLE
    } else if state == NodeState::Current {
        border::ROUNDED
    } else {
        border::PLAIN
    };

    let icon_chars: Vec<char> = node.kind.icon().chars().collect();
    let label_chars: Vec<char> = node.kind.card_label().chars().collect();

    let w = CARD_WIDTH as i32;
    let h = CARD_HEIGHT as i32;
    let last_col = w - 1;
    let last_row = h - 1;
    let inner_w = w - 2;
    let icon_start = 1 + (inner_w - icon_chars.len() as i32) / 2;
    let label_start = 1 + (inner_w - label_chars.len() as i32) / 2;

    let view_x_min = area.x as i32;
    let view_x_max = (area.x + area.width) as i32;
    let view_y_min = area.y as i32;
    let view_y_max = (area.y + area.height) as i32;

    let buf = frame.buffer_mut();
    for row in 0..h {
        let sy = top_y + row;
        if sy < view_y_min || sy >= view_y_max {
            continue;
        }
        for col in 0..w {
            let sx = top_x + col;
            if sx < view_x_min || sx >= view_x_max {
                continue;
            }
            let Some(cell) = buf.cell_mut((sx as u16, sy as u16)) else {
                continue;
            };

            // Border cells
            let border_glyph: Option<&str> = if row == 0 && col == 0 {
                Some(bs.top_left)
            } else if row == 0 && col == last_col {
                Some(bs.top_right)
            } else if row == last_row && col == 0 {
                Some(bs.bottom_left)
            } else if row == last_row && col == last_col {
                Some(bs.bottom_right)
            } else if row == 0 {
                Some(bs.horizontal_top)
            } else if row == last_row {
                Some(bs.horizontal_bottom)
            } else if col == 0 {
                Some(bs.vertical_left)
            } else if col == last_col {
                Some(bs.vertical_right)
            } else {
                None
            };

            if let Some(g) = border_glyph {
                cell.set_symbol(g).set_style(border_style);
                continue;
            }

            // Inner content rows
            if row == 1 {
                let local = col - icon_start;
                if local >= 0 && local < icon_chars.len() as i32 {
                    cell.set_char(icon_chars[local as usize])
                        .set_style(icon_style);
                    continue;
                }
            } else if row == 2 {
                let local = col - label_start;
                if local >= 0 && local < label_chars.len() as i32 {
                    cell.set_char(label_chars[local as usize])
                        .set_style(label_style);
                    continue;
                }
            }

            // Empty inner cell — paint a space so the card stays opaque.
            cell.set_char(' ').set_style(inside_style);
        }
    }
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

fn card_top_left(node: &MapNode, area: Rect, scroll: i32) -> (i32, i32) {
    let (cx, csy) = node_unclipped_position(node, area, scroll);
    let half_w = CARD_WIDTH as i32 / 2;
    let half_h = CARD_HEIGHT as i32 / 2;
    let top_x = cx as i32 - half_w;
    let top_y_screen = csy - half_h;
    let top_y = area.y as i32 + top_y_screen;
    (top_x, top_y)
}

fn card_overlaps_area(top_x: i32, top_y: i32, area: Rect) -> bool {
    let bottom_y = top_y + CARD_HEIGHT as i32;
    let right_x = top_x + CARD_WIDTH as i32;
    let view_x_min = area.x as i32;
    let view_x_max = (area.x + area.width) as i32;
    let view_y_min = area.y as i32;
    let view_y_max = (area.y + area.height) as i32;
    bottom_y > view_y_min && top_y < view_y_max && right_x > view_x_min && top_x < view_x_max
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
