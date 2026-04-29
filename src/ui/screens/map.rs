use crate::environment::{Environment, GroundStyle};
use crate::map::{self, MapGraph, NodeId};
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::SelectScreen;
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

const SKY_BAND_HEIGHT: u16 = 5;

pub struct MapScreen {
    pub graph: MapGraph,
    pub cursor: Option<NodeId>,
    pub environment: Environment,
    pub tick: u32,
    sky_size: (u16, u16),
}

impl MapScreen {
    pub fn new() -> Self {
        let graph = map::generate();
        let cursor = pick_default_cursor(&graph);
        Self {
            graph,
            cursor,
            environment: Environment::generate(80, SKY_BAND_HEIGHT, GroundStyle::default()),
            tick: 0,
            sky_size: (0, 0),
        }
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

    pub fn update(&mut self) {
        let dt = 0.05;
        self.environment.update_cycle(dt, 1.0, 1.0);
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(SKY_BAND_HEIGHT),
                Constraint::Min(8),
                Constraint::Length(6),
            ])
            .split(area);

        let header_area = chunks[0];
        let sky_area = chunks[1];
        let map_area = chunks[2];
        let info_area = chunks[3];

        let sky_size = (sky_area.width, sky_area.height);
        if sky_size != self.sky_size {
            self.environment =
                Environment::generate(sky_size.0, sky_size.1, self.environment.ground_style);
            self.sky_size = sky_size;
        }

        let pulse = pulse_phase(self.tick);
        widgets::render_map_header(frame, &self.graph, header_area);
        widgets::render_environment_background(frame, &self.environment, sky_area);
        widgets::render_map_edges(frame, &self.graph, map_area);
        widgets::render_map_nodes(frame, &self.graph, self.cursor, pulse, map_area);
        widgets::render_map_info(frame, &self.graph, self.cursor, info_area);
    }
}

fn pulse_phase(tick: u32) -> f32 {
    let t = (tick % 24) as f32 / 24.0;
    let s = (t * std::f32::consts::TAU).sin();
    0.5 + 0.5 * s
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
