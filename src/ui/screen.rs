use crate::player::Player;
use crate::ui::screens::{
    AttackPreviewScreen, CatalogueScreen, DemoScreen, FightScreen, GameOverScreen, MapScreen,
    PlaceholderNodeScreen, PlayerInfoScreen, RewardScreen, SelectScreen, SettingsScreen,
    StartScreen, StarterSelectScreen, TransitionScreen,
};
use crossterm::event::KeyCode;
use ratatui::Frame;

pub enum Screen {
    Start(StartScreen),
    StarterSelect(StarterSelectScreen),
    Select(SelectScreen),
    Fight(FightScreen),
    Map(MapScreen),
    PlayerInfo(PlayerInfoScreen),
    Demo(DemoScreen),
    Transition(TransitionScreen),
    AttackPreview(AttackPreviewScreen),
    Settings(SettingsScreen),
    Catalogue(CatalogueScreen),
    Reward(RewardScreen),
    GameOver(GameOverScreen),
    PlaceholderNode(PlaceholderNodeScreen),
}

pub enum Transition {
    Stay,
    Quit,
    Goto(Screen),
}

impl Screen {
    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match self {
            Screen::Start(s) => s.handle_key(key, player),
            Screen::StarterSelect(s) => s.handle_key(key, player),
            Screen::Select(s) => s.handle_key(key, player),
            Screen::Fight(s) => s.handle_key(key, player),
            Screen::Map(s) => s.handle_key(key, player),
            Screen::PlayerInfo(s) => s.handle_key(key, player),
            Screen::Demo(s) => s.handle_key(key, player),
            Screen::Transition(s) => s.handle_key(key, player),
            Screen::AttackPreview(s) => s.handle_key(key, player),
            Screen::Settings(s) => s.handle_key(key, player),
            Screen::Catalogue(s) => s.handle_key(key, player),
            Screen::Reward(s) => s.handle_key(key, player),
            Screen::GameOver(s) => s.handle_key(key, player),
            Screen::PlaceholderNode(s) => s.handle_key(key, player),
        }
    }

    pub fn update(&mut self, player: &mut Player) -> Transition {
        match self {
            Screen::Start(s) => s.update(player),
            Screen::StarterSelect(s) => s.update(player),
            Screen::Select(s) => s.update(player),
            Screen::Fight(s) => s.update(player),
            Screen::Map(s) => s.update(player),
            Screen::PlayerInfo(s) => s.update(player),
            Screen::Demo(s) => s.update(player),
            Screen::Transition(s) => s.update(player),
            Screen::AttackPreview(s) => s.update(player),
            Screen::Settings(s) => s.update(player),
            Screen::Catalogue(s) => s.update(player),
            Screen::Reward(s) => s.update(player),
            Screen::GameOver(s) => s.update(player),
            Screen::PlaceholderNode(s) => s.update(player),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, player: &Player) {
        match self {
            Screen::Start(s) => s.draw(frame, player),
            Screen::StarterSelect(s) => s.draw(frame, player),
            Screen::Select(s) => s.draw(frame, player),
            Screen::Fight(s) => s.draw(frame, player),
            Screen::Map(s) => s.draw(frame, player),
            Screen::PlayerInfo(s) => s.draw(frame, player),
            Screen::Demo(s) => s.draw(frame, player),
            Screen::Transition(s) => s.draw(frame, player),
            Screen::AttackPreview(s) => s.draw(frame, player),
            Screen::Settings(s) => s.draw(frame, player),
            Screen::Catalogue(s) => s.draw(frame, player),
            Screen::Reward(s) => s.draw(frame, player),
            Screen::GameOver(s) => s.draw(frame, player),
            Screen::PlaceholderNode(s) => s.draw(frame, player),
        }
    }
}
