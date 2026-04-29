use crate::map::{self, MapGraph, NodeId};
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::SelectScreen;
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub struct MapScreen {
    pub graph: MapGraph,
    pub cursor: Option<NodeId>,
}

impl MapScreen {
    pub fn new() -> Self {
        let graph = map::generate();
        let cursor = pick_default_cursor(&graph);
        Self { graph, cursor }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Transition::Goto(Screen::Select(SelectScreen::new()));
            }
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1),
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(-1),
            KeyCode::Enter => {
                if let Some(id) = self.cursor {
                    if self.graph.select(id) {
                        self.cursor = pick_default_cursor(&self.graph);
                    }
                }
            }
            _ => {}
        }
        Transition::Stay
    }

    fn move_cursor(&mut self, delta: i32) {
        let reachable = sorted_reachable(&self.graph);
        if reachable.is_empty() {
            self.cursor = None;
            return;
        }
        let len = reachable.len() as i32;
        let pos = self
            .cursor
            .and_then(|id| reachable.iter().position(|&n| n == id))
            .unwrap_or(0) as i32;
        let new_pos = (pos + delta).rem_euclid(len) as usize;
        self.cursor = Some(reachable[new_pos]);
    }

    pub fn update(&mut self) {}

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(6),
            ])
            .split(area);
        widgets::render_map_header(frame, &self.graph, chunks[0]);
        widgets::render_map_edges(frame, &self.graph, chunks[1]);
        widgets::render_map_nodes(frame, &self.graph, self.cursor, chunks[1]);
        widgets::render_map_info(frame, &self.graph, self.cursor, chunks[2]);
    }
}

fn sorted_reachable(graph: &MapGraph) -> Vec<NodeId> {
    let mut ids = graph.reachable();
    ids.sort_by_key(|&id| graph.node(id).column);
    ids
}

fn pick_default_cursor(graph: &MapGraph) -> Option<NodeId> {
    let reachable = sorted_reachable(graph);
    if reachable.is_empty() {
        return None;
    }
    Some(reachable[reachable.len() / 2])
}
