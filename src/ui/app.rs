use crate::crab::Crab;
use crate::ui::widgets;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Frame;
use std::time::{Duration, Instant};

pub struct App {
    pub should_quit: bool,
    pub crab: Crab,
    last_terminal_size: (u16, u16),
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            crab: Crab::new((10.0, 100.0), 95),
            last_terminal_size: (0, 0),
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
                        self.handle_key(key.code);
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.update();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    fn update(&mut self) {
        let dt = 0.05;
        let bounds = (
            self.last_terminal_size.0 as f32 - 2.0,
            self.last_terminal_size.1 as f32,
        );
        if bounds.0 > 0.0 && bounds.1 > 0.0 {
            self.crab.update(dt, bounds);
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.last_terminal_size = (area.width, area.height);
        widgets::render_crab(frame, &self.crab, area);
    }
}
