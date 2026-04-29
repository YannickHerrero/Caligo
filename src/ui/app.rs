use crate::crab::Crab;
use crate::environment::{Environment, GroundStyle};
use crate::fight::FightState;
use crate::ui::widgets;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;
use std::time::{Duration, Instant};

pub struct App {
    pub should_quit: bool,
    pub crab: Crab,
    pub environment: Environment,
    pub fight: FightState,
    last_terminal_size: (u16, u16),
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            crab: Crab::new((10.0, 100.0), 95),
            environment: Environment::generate(80, 15, GroundStyle::default()),
            fight: FightState::new(),
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
        self.environment.update_cycle(dt, 1.0, 1.0);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(8),
            ])
            .split(area);

        let top_bar_area = chunks[0];
        let scene_area = chunks[1];
        let _action_area = chunks[2];

        let current_size = (scene_area.width, scene_area.height);
        if current_size != self.last_terminal_size {
            self.environment = Environment::generate(
                scene_area.width,
                scene_area.height,
                self.environment.ground_style,
            );
            self.last_terminal_size = current_size;
        }

        widgets::render_top_bar(frame, &self.fight, top_bar_area);
        widgets::render_environment_background(frame, &self.environment, scene_area);
        widgets::render_crab(frame, &self.crab, scene_area);
        widgets::render_enemy(frame, &self.fight.enemy, scene_area);
        widgets::render_ground(frame, &self.environment, scene_area);
        widgets::render_hp_bars(frame, &self.fight, scene_area);
    }
}
