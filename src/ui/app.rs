use crate::crab::Crab;
use crate::environment::{Environment, GroundStyle};
use crate::fight::{Action, Animation, FightState, MenuState};
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
    last_action_height: u16,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            crab: Crab::new((6.0, 100.0), 95),
            environment: Environment::generate(80, 15, GroundStyle::default()),
            fight: FightState::new(),
            last_terminal_size: (0, 0),
            last_action_height: 0,
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
        match self.fight.menu_state {
            MenuState::Main => self.handle_main_menu(key),
            MenuState::AttackSelect => self.handle_attack_menu(key),
            MenuState::ItemSelect => self.handle_item_menu(key),
        }
    }

    fn handle_main_menu(&mut self, key: KeyCode) {
        let action_count = Action::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                self.fight.selected_action =
                    (self.fight.selected_action + action_count - 1) % action_count;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.fight.selected_action = (self.fight.selected_action + 1) % action_count;
            }
            KeyCode::Enter => match self.fight.selected() {
                Action::Attack => {
                    self.fight.menu_state = MenuState::AttackSelect;
                }
                Action::Item => {
                    self.fight.menu_state = MenuState::ItemSelect;
                }
                Action::Flee => {}
            },
            _ => {}
        }
    }

    fn handle_attack_menu(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                self.fight.menu_state = MenuState::Main;
            }
            KeyCode::Char('q') => {
                self.fight.menu_state = MenuState::Main;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.fight.attack_selected ^= 1;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.fight.attack_selected ^= 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.fight.attack_selected >= 2 {
                    self.fight.attack_selected -= 2;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.fight.attack_selected < 2 {
                    self.fight.attack_selected += 2;
                }
            }
            KeyCode::Enter => {
                self.start_attack_animation();
            }
            _ => {}
        }
    }

    fn start_attack_animation(&mut self) {
        let idx = self.fight.attack_selected;
        if idx >= self.fight.attacks.len() {
            return;
        }
        let kind = self.fight.attacks[idx].kind;
        let start_x = self.crab.position.0;
        let target_x = (self.last_terminal_size.0 as f32 - 18.0).max(start_x + 5.0);
        self.fight.animation = Some(Animation::new(kind, start_x, target_x));
        self.fight.menu_state = MenuState::Main;
    }

    fn handle_item_menu(&mut self, key: KeyCode) {
        let len = self.fight.items.len();
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                self.fight.menu_state = MenuState::Main;
            }
            KeyCode::Char('q') => {
                self.fight.menu_state = MenuState::Main;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.fight.item_selected > 0 {
                    self.fight.item_selected -= 1;
                    if self.fight.item_selected < self.fight.item_scroll {
                        self.fight.item_scroll = self.fight.item_selected;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.fight.item_selected + 1 < len {
                    self.fight.item_selected += 1;
                    let visible = self.last_action_height as usize;
                    if visible > 0 && self.fight.item_selected >= self.fight.item_scroll + visible {
                        self.fight.item_scroll = self.fight.item_selected + 1 - visible;
                    }
                }
            }
            KeyCode::Enter => {}
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
            self.crab.walk_range_x = Some((0.0, bounds.0 * 0.4));
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
        let action_area = chunks[2];
        self.last_action_height = action_area.height.saturating_sub(2);

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
        match self.fight.menu_state {
            MenuState::Main => widgets::render_action_menu(frame, &self.fight, action_area),
            MenuState::AttackSelect => {
                widgets::render_attack_menu(frame, &self.fight, action_area)
            }
            MenuState::ItemSelect => widgets::render_item_menu(frame, &self.fight, action_area),
        }
    }
}
