use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::SelectScreen;
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::Frame;
use std::time::{Duration, Instant};

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    pub player: Player,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            screen: Screen::Select(SelectScreen::new()),
            player: Player::new(),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(50);
        let mut last_tick = Instant::now();

        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match self.screen.handle_key(key.code, &mut self.player) {
                            Transition::Stay => {}
                            Transition::Quit => self.should_quit = true,
                            Transition::Goto(screen) => self.screen = screen,
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                match self.screen.update(&mut self.player) {
                    Transition::Stay => {}
                    Transition::Quit => self.should_quit = true,
                    Transition::Goto(screen) => self.screen = screen,
                }
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.screen.draw(frame, &self.player);
    }
}
