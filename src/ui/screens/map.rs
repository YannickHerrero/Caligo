use crate::map::{self, MapGraph};
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::SelectScreen;
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct MapScreen {
    pub graph: MapGraph,
}

impl MapScreen {
    pub fn new() -> Self {
        Self {
            graph: map::generate(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                Transition::Goto(Screen::Select(SelectScreen::new()))
            }
            _ => Transition::Stay,
        }
    }

    pub fn update(&mut self) {}

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        widgets::render_map_nodes(frame, &self.graph, area);
    }
}
