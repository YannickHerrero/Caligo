use crate::ui::screens::FightScreen;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub enum Screen {
    Fight(FightScreen),
}

pub enum Transition {
    Stay,
    Quit,
    Goto(Screen),
}

impl Screen {
    pub fn handle_key(&mut self, key: KeyCode) -> Transition {
        match self {
            Screen::Fight(s) => s.handle_key(key),
        }
    }

    pub fn update(&mut self) {
        match self {
            Screen::Fight(s) => s.update(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self {
            Screen::Fight(s) => s.draw(frame),
        }
    }
}
