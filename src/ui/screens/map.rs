use crate::map::{self, MapGraph, NodeId};
use crate::player::Player;
use crate::run::Run;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{
    FightScreen, PlayerInfoScreen, SelectScreen, TransitionKind, TransitionScreen,
};
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

const SCROLL_STEP: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMenuState {
    Browsing,
    Confirming,
}

pub struct MapScreen {
    pub run: Run,
    pub cursor: Option<NodeId>,
    pub tick: u32,
    pub menu_state: MapMenuState,
    scroll: i32,
    last_viewport_height: u16,
}

impl MapScreen {
    pub fn new() -> Self {
        Self::with_run(Run::new(map::generate()))
    }

    pub fn with_run(run: Run) -> Self {
        let cursor = pick_default_cursor(&run.map);
        Self {
            run,
            cursor,
            tick: 0,
            menu_state: MapMenuState::Browsing,
            scroll: 0,
            last_viewport_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        if matches!(key, KeyCode::Tab) && self.menu_state == MapMenuState::Browsing {
            let from = std::mem::replace(self, MapScreen::new());
            return Transition::Goto(Screen::PlayerInfo(PlayerInfoScreen::new(from)));
        }
        match self.menu_state {
            MapMenuState::Browsing => self.handle_browsing(key),
            MapMenuState::Confirming => self.handle_confirming(key, player),
        }
    }

    fn handle_browsing(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Transition::Goto(Screen::Select(SelectScreen::new()));
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_cursor(-1);
                self.center_scroll_on_cursor();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_cursor(1);
                self.center_scroll_on_cursor();
            }
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-SCROLL_STEP),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(SCROLL_STEP),
            KeyCode::PageUp => self.scroll_by(-(self.last_viewport_height as i32)),
            KeyCode::PageDown => self.scroll_by(self.last_viewport_height as i32),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = self.max_scroll(),
            KeyCode::Enter => {
                if self.cursor.is_some() {
                    self.menu_state = MapMenuState::Confirming;
                    self.center_scroll_on_cursor();
                }
            }
            _ => {}
        }
        Transition::Stay
    }

    fn handle_confirming(&mut self, key: KeyCode, player: &Player) -> Transition {
        match key {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                self.menu_state = MapMenuState::Browsing;
                Transition::Stay
            }
            KeyCode::Enter => {
                let Some(id) = self.cursor else {
                    self.menu_state = MapMenuState::Browsing;
                    return Transition::Stay;
                };
                let kind = self.run.map.node(id).kind;
                if !self.run.map.select(id) {
                    self.menu_state = MapMenuState::Browsing;
                    return Transition::Stay;
                }
                self.menu_state = MapMenuState::Browsing;
                let from = std::mem::replace(self, MapScreen::new());
                let transition = TransitionScreen::new(
                    Screen::Map(from),
                    Screen::Fight(FightScreen::new(player)),
                    TransitionKind::from(kind),
                );
                Transition::Goto(Screen::Transition(transition))
            }
            _ => Transition::Stay,
        }
    }

    fn scroll_by(&mut self, delta: i32) {
        let max = self.max_scroll();
        self.scroll = (self.scroll + delta).clamp(0, max);
    }

    fn max_scroll(&self) -> i32 {
        let total = widgets::map_virtual_height();
        (total - self.last_viewport_height as i32).max(0)
    }

    fn center_scroll_on_cursor(&mut self) {
        if self.last_viewport_height == 0 {
            return;
        }
        self.scroll =
            widgets::compute_map_scroll(&self.run.map, self.cursor, self.last_viewport_height);
    }

    fn move_cursor(&mut self, delta: i32) {
        let reachable = sorted_reachable(&self.run.map);
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

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        self.tick = self.tick.wrapping_add(1);
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(6),
            ])
            .split(area);

        let header_area = chunks[0];
        let map_area = chunks[1];
        let info_area = chunks[2];

        let pulse = pulse_phase(self.tick);
        if self.last_viewport_height != map_area.height {
            self.last_viewport_height = map_area.height;
            self.center_scroll_on_cursor();
        }
        // Re-clamp in case viewport shrank below current scroll.
        let max = self.max_scroll();
        if self.scroll > max {
            self.scroll = max;
        }
        let scroll = self.scroll;
        widgets::render_map_header(frame, &self.run.map, header_area);
        widgets::render_map_edges(frame, &self.run.map, scroll, map_area);
        widgets::render_map_nodes(frame, &self.run.map, self.cursor, pulse, scroll, map_area);
        widgets::render_map_info(frame, &self.run.map, self.cursor, info_area);
        if self.menu_state == MapMenuState::Confirming {
            if let Some(id) = self.cursor {
                widgets::render_map_confirm(frame, self.run.map.node(id), area);
            }
        }
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
