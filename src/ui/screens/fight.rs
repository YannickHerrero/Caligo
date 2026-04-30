use crate::crab::Crab;
use crate::data::attacks as attack_lib;
use crate::environment::{Environment, GroundStyle};
use crate::fight::{Action, Animation, Enemy, FightState, MenuState};
use crate::map::NodeKind;
use crate::player::Player;
use rand::seq::SliceRandom;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::reward::{apply_rewards, roll_rewards};
use crate::ui::screens::{GameOverScreen, MapScreen, RewardScreen, SelectScreen, VictoryScreen};
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub struct FightScreen {
    pub crab: Crab,
    pub environment: Environment,
    pub fight: FightState,
    /// The map screen we entered from. `Some` for fights launched from a
    /// real run; `None` for the standalone --debug fight.
    pub map: Option<Box<MapScreen>>,
    /// Which kind of map node this fight came from. Drives flee
    /// permissions and reward shape. `None` for standalone --debug fights.
    pub node_kind: Option<NodeKind>,
    /// Set to true when the player has triggered an outcome that should
    /// end the fight on the next update tick (lets the in-flight animation
    /// finish before the screen hands off).
    pending_exit: Option<FightOutcome>,
    /// True once the player has committed to and resolved their action
    /// for the current round. Reset on round reroll.
    player_acted: bool,
    /// True once the enemy has resolved its action for the current round.
    /// Reset on round reroll.
    enemy_acted: bool,
    last_terminal_size: (u16, u16),
    last_action_height: u16,
}

#[derive(Debug, Clone, Copy)]
enum FightOutcome {
    Victory,
    Defeat,
    Flee,
}

impl FightScreen {
    pub fn new(player: &Player) -> Self {
        Self {
            crab: Crab::new((6.0, 100.0), 95),
            environment: Environment::generate(80, 15, GroundStyle::default()),
            fight: FightState::from_player(player),
            map: None,
            node_kind: None,
            pending_exit: None,
            player_acted: false,
            enemy_acted: false,
            last_terminal_size: (0, 0),
            last_action_height: 0,
        }
    }

    /// Variant entered from a real map node — the fight carries the map
    /// forward so it can hand control back when the fight ends.
    pub fn from_map(
        player: &Player,
        map: Box<MapScreen>,
        enemy: Enemy,
        node_kind: NodeKind,
    ) -> Self {
        let mut fight = FightState::from_player_with_enemy(player, enemy);
        fight.player_type = Some(map.run.starter.primary_type);
        let mut rng = rand::thread_rng();
        fight.roll_round_order(&mut rng);
        Self {
            crab: Crab::new((6.0, 100.0), 95),
            environment: Environment::generate(80, 15, GroundStyle::default()),
            fight,
            map: Some(map),
            node_kind: Some(node_kind),
            pending_exit: None,
            player_acted: false,
            enemy_acted: false,
            last_terminal_size: (0, 0),
            last_action_height: 0,
        }
    }

    fn flee_allowed(&self) -> bool {
        matches!(
            self.node_kind,
            None | Some(NodeKind::EasyFight) | Some(NodeKind::NormalFight)
        )
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        if self.fight.animation.is_some() || self.fight.message.is_some() {
            return Transition::Stay;
        }
        match self.fight.menu_state {
            MenuState::Main => self.handle_main_menu(key),
            MenuState::AttackSelect => self.handle_attack_menu(key),
            MenuState::ItemSelect => self.handle_item_menu(key),
        }
    }

    fn handle_main_menu(&mut self, key: KeyCode) -> Transition {
        let action_count = Action::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                return self.exit_fight();
            }
            // Debug: force defeat. Lets the death flow be exercised while
            // real combat is stubbed.
            KeyCode::Char('L') => {
                self.pending_exit = Some(FightOutcome::Defeat);
            }
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
                Action::Flee => {
                    if self.flee_allowed() {
                        self.pending_exit = Some(FightOutcome::Flee);
                    }
                }
            },
            _ => {}
        }
        Transition::Stay
    }

    fn handle_attack_menu(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                self.fight.menu_state = MenuState::Main;
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
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
        Transition::Stay
    }

    /// Exit a fight without victory: return to the carried map if there is
    /// one, otherwise fall back to SelectScreen (debug entry).
    fn exit_fight(&mut self) -> Transition {
        match self.map.take() {
            Some(map) => Transition::Goto(Screen::Map(*map)),
            None => Transition::Goto(Screen::Select(SelectScreen::new())),
        }
    }

    fn start_attack_animation(&mut self) {
        let idx = self.fight.attack_selected;
        if idx >= self.fight.attacks.len() {
            return;
        }
        let attack = self.fight.attacks[idx].clone();
        let start_x = self.crab.position.0;
        let target_x = (self.last_terminal_size.0 as f32 - 18.0).max(start_x + 5.0);
        self.fight
            .set_message(format!("You used {}!", attack.name), 0.6);
        self.fight.animation = Some(Animation::for_attack(&attack, start_x, target_x));
        self.fight.pending_player_attack = Some(attack);
        self.fight.menu_state = MenuState::Main;
        self.player_acted = true;
    }

    /// Pick a uniform-random move from the enemy moveset, telegraph it,
    /// and store the resolved Attack so the impact lands once the message
    /// clears.
    fn queue_enemy_attack(&mut self) {
        let mut rng = rand::thread_rng();
        let Some(name) = self.fight.enemy.moveset.choose(&mut rng) else {
            // Empty moveset — skip the enemy's turn rather than stalling.
            self.enemy_acted = true;
            return;
        };
        let Some(attack) = attack_lib::find_by_name(name) else {
            self.enemy_acted = true;
            return;
        };
        let enemy_name = self.fight.enemy.name.clone();
        self.fight
            .set_message(format!("{} used {}!", enemy_name, attack.name), 0.7);
        self.fight.pending_enemy_attack = Some(attack);
        self.enemy_acted = true;
    }

    fn handle_item_menu(&mut self, key: KeyCode) -> Transition {
        let len = self.fight.items.len();
        match key {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
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
        Transition::Stay
    }

    pub fn update(&mut self, player: &mut Player) -> Transition {
        let dt = 0.05;
        let bounds = (
            self.last_terminal_size.0 as f32 - 2.0,
            self.last_terminal_size.1 as f32,
        );

        if let Some(anim) = self.fight.animation.as_mut() {
            anim.tick(dt);
            if anim.is_done() {
                self.fight.animation = None;
                if let Some(attack) = self.fight.pending_player_attack.take() {
                    let mut rng = rand::thread_rng();
                    let damage = self.fight.resolve_player_attack(&attack, &mut rng);
                    let enemy_name = self.fight.enemy.name.clone();
                    let msg = if self.fight.enemy.hp == 0 {
                        format!("{} fainted!", enemy_name)
                    } else if damage > 0 {
                        format!("{} took {} damage!", enemy_name, damage)
                    } else {
                        format!("It had no effect on {}.", enemy_name)
                    };
                    self.fight.set_message(msg, 1.0);
                    if self.fight.enemy.hp == 0 {
                        self.pending_exit = Some(FightOutcome::Victory);
                    }
                }
            }
        } else if bounds.0 > 0.0 && bounds.1 > 0.0 {
            self.crab.walk_range_x = Some((0.0, bounds.0 * 0.4));
            self.crab.update(dt, bounds);
        }

        self.environment.update_cycle(dt, 1.0, 1.0);
        self.fight.tick_message(dt);

        // Apply the enemy's chosen attack once its telegraph message has
        // cleared. This is the "impact" step of the enemy turn.
        if self.fight.animation.is_none() && self.fight.message.is_none() {
            if let Some(attack) = self.fight.pending_enemy_attack.take() {
                let mut rng = rand::thread_rng();
                let damage = self.fight.resolve_enemy_attack(&attack, &mut rng);
                let msg = if self.fight.player_hp == 0 {
                    "You fainted!".to_string()
                } else if damage > 0 {
                    format!("You took {} damage!", damage)
                } else {
                    "It had no effect.".to_string()
                };
                self.fight.set_message(msg, 1.0);
                if self.fight.player_hp == 0 {
                    self.pending_exit = Some(FightOutcome::Defeat);
                }
            }
        }

        // Round-loop driver: only runs when nothing is in-flight (no
        // animation, no message, no pending action, no end-of-fight).
        let idle = self.pending_exit.is_none()
            && self.fight.animation.is_none()
            && self.fight.message.is_none()
            && self.fight.pending_enemy_attack.is_none()
            && self.fight.pending_player_attack.is_none()
            && self.fight.menu_state == MenuState::Main
            && self.fight.enemy.hp > 0
            && self.fight.player_hp > 0;

        if idle {
            if self.player_acted && self.enemy_acted {
                // Round complete; reroll for the next round.
                let mut rng = rand::thread_rng();
                self.fight.roll_round_order(&mut rng);
                self.player_acted = false;
                self.enemy_acted = false;
            } else if !self.enemy_acted
                && (self.fight.enemy_first_this_round || self.player_acted)
            {
                // Enemy's turn this round.
                self.queue_enemy_attack();
            }
            // Otherwise: it's the player's turn. We just wait for input.
        }

        self.fight.commit_to_player(player);

        // Wait for any in-flight animation AND the linger of the result
        // message to clear before handing off — otherwise the player never
        // gets to see "X fainted!".
        if self.fight.animation.is_none() && self.fight.message.is_none() {
            if let Some(outcome) = self.pending_exit.take() {
                return self.resolve_outcome(outcome, player);
            }
        }
        Transition::Stay
    }

    fn resolve_outcome(&mut self, outcome: FightOutcome, player: &mut Player) -> Transition {
        match outcome {
            FightOutcome::Victory => self.victory(player),
            FightOutcome::Flee => self.exit_fight(),
            FightOutcome::Defeat => self.defeat(player),
        }
    }

    fn defeat(&mut self, player: &Player) -> Transition {
        let Some(map) = self.map.take() else {
            return self.exit_fight();
        };
        let starter = map.run.starter.clone();
        let floor_reached = map
            .run
            .map
            .current
            .map(|id| map.run.map.node(id).floor as u32 + 1)
            .unwrap_or(0);
        Transition::Goto(Screen::GameOver(GameOverScreen::new(
            starter,
            floor_reached,
            player.gold,
        )))
    }

    fn victory(&mut self, player: &mut Player) -> Transition {
        let (Some(map), Some(kind)) = (self.map.take(), self.node_kind) else {
            // Standalone fight without a run — nothing to reward.
            return self.exit_fight();
        };
        let mut rng = rand::thread_rng();
        let (gold, items) = roll_rewards(kind, &mut rng);
        apply_rewards(player, gold, &items);

        if matches!(kind, NodeKind::Boss) {
            let starter = map.run.starter.clone();
            let floor_reached = map
                .run
                .map
                .current
                .map(|id| map.run.map.node(id).floor as u32 + 1)
                .unwrap_or(0);
            return Transition::Goto(Screen::Victory(VictoryScreen::new(
                starter,
                floor_reached,
                player.gold,
                gold,
                items,
            )));
        }

        Transition::Goto(Screen::Reward(RewardScreen::new(map, gold, items, kind)))
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
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
        let crab_override = self
            .fight
            .animation
            .as_ref()
            .map(|anim| anim.crab_position(self.crab.position));

        widgets::render_environment_background(frame, &self.environment, scene_area);
        widgets::render_crab(frame, &self.crab, scene_area, crab_override);
        widgets::render_enemy(frame, &self.fight.enemy, scene_area);
        if let Some(anim) = self.fight.animation.as_ref() {
            widgets::render_projectile(frame, anim, self.crab.position.1, scene_area);
            widgets::render_particles(frame, anim, self.crab.position, scene_area);
        }
        widgets::render_ground(frame, &self.environment, scene_area);
        widgets::render_hp_bars(frame, &self.fight, scene_area);
        // While a turn is resolving (animation in flight or message lingering)
        // the message strip replaces the action menu.
        if let Some(msg) = self.fight.message.as_deref() {
            widgets::render_message_strip(frame, msg, action_area);
        } else if self.fight.animation.is_none() {
            match self.fight.menu_state {
                MenuState::Main => widgets::render_action_menu(frame, &self.fight, action_area),
                MenuState::AttackSelect => {
                    widgets::render_attack_menu(frame, &self.fight, action_area)
                }
                MenuState::ItemSelect => {
                    widgets::render_item_menu(frame, &self.fight, action_area)
                }
            }
        }
    }
}
