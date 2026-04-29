use crate::ui::screens::{DemoScreen, FightScreen, MapScreen, SelectScreen, TransitionScreen};
use crossterm::event::KeyCode;
use ratatui::Frame;

pub enum Screen {
    Select(SelectScreen),
    Fight(FightScreen),
    Map(MapScreen),
    Demo(DemoScreen),
    Transition(TransitionScreen),
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
            Screen::Demo(s) => s.handle_key(key),
            Screen::Transition(s) => s.handle_key(key),
        }
    }

    pub fn update(&mut self) -> Transition {
        match self {
            Screen::Select(s) => s.update(),
            Screen::Fight(s) => s.update(),
            Screen::Map(s) => s.update(),
            Screen::Demo(s) => s.update(),
            Screen::Transition(s) => s.update(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self {
            Screen::Select(s) => s.draw(frame),
            Screen::Fight(s) => s.draw(frame),
            Screen::Map(s) => s.draw(frame),
            Screen::Demo(s) => s.draw(frame),
            Screen::Transition(s) => s.draw(frame),
        }
    }
}
