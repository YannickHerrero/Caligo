use crate::ui::screens::{FightScreen, MapScreen, SelectScreen};
use crossterm::event::KeyCode;
use ratatui::Frame;

pub enum Screen {
    Select(SelectScreen),
    Fight(FightScreen),
    Map(MapScreen),
}

pub enum Transition {
    Stay,
    Quit,
    Goto(Screen),
}

impl Screen {
    pub fn handle_key(&mut self, key: KeyCode) -> Transition {
        match self {
            Screen::Select(s) => s.handle_key(key),
            Screen::Fight(s) => s.handle_key(key),
            Screen::Map(s) => s.handle_key(key),
        }
    }

    pub fn update(&mut self) {
        match self {
            Screen::Select(s) => s.update(),
            Screen::Fight(s) => s.update(),
            Screen::Map(s) => s.update(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self {
            Screen::Select(s) => s.draw(frame),
            Screen::Fight(s) => s.draw(frame),
            Screen::Map(s) => s.draw(frame),
        }
    }
}
