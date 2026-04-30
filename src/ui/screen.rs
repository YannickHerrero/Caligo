use crate::player::Player;
use crate::ui::screens::{
    AttackPreviewScreen, DemoScreen, FightScreen, MapScreen, PlayerInfoScreen, SelectScreen,
    SettingsScreen, TransitionScreen,
};
use crossterm::event::KeyCode;
use ratatui::Frame;

pub enum Screen {
    Select(SelectScreen),
    Fight(FightScreen),
    Map(MapScreen),
    PlayerInfo(PlayerInfoScreen),
    Demo(DemoScreen),
    Transition(TransitionScreen),
    AttackPreview(AttackPreviewScreen),
    Settings(SettingsScreen),
}

pub enum Transition {
    Stay,
    Quit,
    Goto(Screen),
}

impl Screen {
    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match self {
            Screen::Select(s) => s.handle_key(key, player),
            Screen::Fight(s) => s.handle_key(key, player),
            Screen::Map(s) => s.handle_key(key, player),
            Screen::PlayerInfo(s) => s.handle_key(key, player),
            Screen::Demo(s) => s.handle_key(key, player),
            Screen::Transition(s) => s.handle_key(key, player),
            Screen::AttackPreview(s) => s.handle_key(key, player),
            Screen::Settings(s) => s.handle_key(key, player),
        }
    }

    pub fn update(&mut self, player: &mut Player) -> Transition {
        match self {
            Screen::Select(s) => s.update(player),
            Screen::Fight(s) => s.update(player),
            Screen::Map(s) => s.update(player),
            Screen::PlayerInfo(s) => s.update(player),
            Screen::Demo(s) => s.update(player),
            Screen::Transition(s) => s.update(player),
            Screen::AttackPreview(s) => s.update(player),
            Screen::Settings(s) => s.update(player),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, player: &Player) {
        match self {
            Screen::Select(s) => s.draw(frame, player),
            Screen::Fight(s) => s.draw(frame, player),
            Screen::Map(s) => s.draw(frame, player),
            Screen::PlayerInfo(s) => s.draw(frame, player),
            Screen::Demo(s) => s.draw(frame, player),
            Screen::Transition(s) => s.draw(frame, player),
            Screen::AttackPreview(s) => s.draw(frame, player),
            Screen::Settings(s) => s.draw(frame, player),
        }
    }
}
