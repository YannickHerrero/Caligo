use crate::crab::Crab;
use crate::data::attacks as attack_lib;
use crate::environment::{Environment, GroundStyle};
use crate::fight::{Action, Animation, Enemy, FightState, Item, MenuState, PotionSize};
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
    /// The second action of the current round, queued to fire once the
    /// first action finishes resolving. `None` between rounds.
    next_action: Option<QueuedAction>,
    /// True from the moment the player commits an action this round
    /// until both queued actions have finished. Drives end-of-round
    /// bookkeeping (tick buffs once, not per-frame).
    round_active: bool,
    /// HP at the moment the fight started — restored on Revive Pearl
    /// consumption.
    pre_fight_hp: u32,
    /// MP at the moment the fight started — restored on Revive Pearl
    /// consumption.
    pre_fight_mana: u32,
    /// True the moment the player drops to 0 HP if they're carrying a
    /// Revive Pearl. The actual prompt opens after the impact message
    /// clears.
    revive_prompt_pending: bool,
    /// True while the Revive Pearl prompt is on screen. Locks input to Y/N.
    revive_prompt_open: bool,
    /// True after the active member faints AND there's at least one
    /// other alive member. Drives the forced member-select popup that
    /// opens once the impact message clears.
    faint_swap_pending: bool,
    /// State for the member-select popup. None when not active.
    /// `forced` blocks Esc/cancel (set when invoked by a faint).
    switch_prompt: Option<SwitchPromptState>,
    last_terminal_size: (u16, u16),
    last_action_height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Actor {
    Player,
    Enemy,
}

#[derive(Debug, Clone)]
struct QueuedAction {
    actor: Actor,
    attack: crate::fight::Attack,
}

#[derive(Debug, Clone)]
struct SwitchPromptState {
    /// Index into `Run.party` highlighted for swap-in.
    selected: usize,
    /// True when the prompt was opened by a faint — Esc / cancel doesn't
    /// dismiss it.
    forced: bool,
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
            next_action: None,
            round_active: false,
            pre_fight_hp: player.hp,
            pre_fight_mana: player.mana,
            revive_prompt_pending: false,
            revive_prompt_open: false,
            faint_swap_pending: false,
            switch_prompt: None,
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
        let active = map.run.active_member();
        fight.player_type = Some(active.template.primary_type);
        // Permanent meta boost: read directly off the active member so
        // switching mid-fight automatically updates the multiplier.
        fight.player_attack_boost_pct = active.attack_boost_pct;
        fight.active_member_name = active.template.name.clone();
        Self {
            crab: Crab::new((6.0, 100.0), 95),
            environment: Environment::generate(80, 15, GroundStyle::default()),
            fight,
            map: Some(map),
            node_kind: Some(node_kind),
            pending_exit: None,
            next_action: None,
            round_active: false,
            pre_fight_hp: player.hp,
            pre_fight_mana: player.mana,
            revive_prompt_pending: false,
            revive_prompt_open: false,
            faint_swap_pending: false,
            switch_prompt: None,
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

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        if self.revive_prompt_open {
            return self.handle_revive_prompt(key, player);
        }
        if self.switch_prompt.is_some() {
            return self.handle_switch_prompt(key, player);
        }
        if self.fight.animation.is_some() || self.fight.message.is_some() {
            return Transition::Stay;
        }
        match self.fight.menu_state {
            MenuState::Main => self.handle_main_menu(key, player),
            MenuState::AttackSelect => self.handle_attack_menu(key),
            MenuState::ItemSelect => self.handle_item_menu(key),
        }
    }

    fn handle_revive_prompt(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                consume_one_revive_pearl(&mut self.fight.items);
                self.fight.player_hp = self.pre_fight_hp.max(1);
                self.fight.player_mana = self.pre_fight_mana;
                self.revive_prompt_open = false;
                self.revive_prompt_pending = false;
                // Sync now so the restored HP/MP and reduced pearl count
                // are carried back to the Player when we leave the screen.
                self.fight.commit_to_player(player);
                // And sync the Player back into the active party
                // member, so the resurrected HP/MP persists into the
                // next fight.
                self.commit_active_member(player);
                self.exit_fight()
            }
            KeyCode::Char('n')
            | KeyCode::Char('N')
            | KeyCode::Esc
            | KeyCode::Char('q') => {
                self.revive_prompt_open = false;
                if !self.alive_other_members().is_empty() {
                    // Other members can still fight — pop the forced
                    // member-select instead of ending the run.
                    self.switch_prompt = Some(SwitchPromptState {
                        selected: self.first_alive_other(),
                        forced: true,
                    });
                } else {
                    self.pending_exit = Some(FightOutcome::Defeat);
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn handle_main_menu(&mut self, key: KeyCode, player: &Player) -> Transition {
        let action_count = Action::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.commit_active_member(player);
                return self.exit_fight();
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
                Action::Switch => {
                    if self.alive_other_members().is_empty() {
                        self.fight
                            .set_message("No other members able to fight.", 0.8);
                    } else {
                        self.switch_prompt = Some(SwitchPromptState {
                            selected: self.first_alive_other(),
                            forced: false,
                        });
                    }
                }
                Action::Flee => {
                    if self.flee_allowed() {
                        self.fight.set_message("You fled!", 0.8);
                        self.pending_exit = Some(FightOutcome::Flee);
                    } else {
                        self.fight
                            .set_message("You can't flee from this fight!", 0.8);
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
                self.commit_player_attack();
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

    /// Write the player's working HP/MP/attacks back into the run's
    /// active party member so they persist into the next fight (or end
    /// the run if 0 HP).
    fn commit_active_member(&mut self, player: &Player) {
        if let Some(map) = self.map.as_mut() {
            let idx = map.run.active;
            if let Some(member) = map.run.party.get_mut(idx) {
                player.sync_to_member(member);
            }
        }
    }

    /// Indices of party members other than the active one that still
    /// have HP > 0.
    fn alive_other_members(&self) -> Vec<usize> {
        let Some(map) = self.map.as_ref() else {
            return Vec::new();
        };
        let active = map.run.active;
        map.run
            .party
            .iter()
            .enumerate()
            .filter(|(idx, m)| *idx != active && m.current_hp > 0)
            .map(|(idx, _)| idx)
            .collect()
    }

    fn first_alive_other(&self) -> usize {
        self.alive_other_members().first().copied().unwrap_or(0)
    }

    /// Swap the run's active member to `slot`. Costs the player's turn
    /// (Pokemon-rules) so the round logic should fire the enemy's
    /// response after this returns.
    fn perform_switch(&mut self, slot: usize, player: &mut Player) {
        let Some(map) = self.map.as_mut() else {
            return;
        };
        if slot >= map.run.party.len() || slot == map.run.active {
            return;
        }
        // Sync working state back to the outgoing member.
        let outgoing_idx = map.run.active;
        if let Some(outgoing) = map.run.party.get_mut(outgoing_idx) {
            player.sync_to_member(outgoing);
        }
        // Swap active.
        map.run.active = slot;
        // Pull the new member's stats into Player.
        let incoming = map.run.party[slot].clone();
        player.sync_from_member(&incoming);
        // Reflect the new active in fight state too.
        self.fight.player_hp = player.hp;
        self.fight.player_max_hp = player.max_hp();
        self.fight.player_mana = player.mana;
        self.fight.player_max_mana = player.max_mana();
        self.fight.player_type = Some(incoming.template.primary_type);
        self.fight.player_attack_boost_pct = incoming.attack_boost_pct;
        self.fight.active_member_name = incoming.template.name.clone();
        self.fight.attacks = incoming.attacks.clone();
        self.fight.attack_selected = 0;
        // Reset the active member's hit-flash so the new sprite isn't
        // mid-blink.
        self.fight.player_hit_remaining = 0.0;
    }

    fn handle_switch_prompt(
        &mut self,
        key: KeyCode,
        player: &mut Player,
    ) -> Transition {
        let Some(prompt) = self.switch_prompt.clone() else {
            return Transition::Stay;
        };
        let alive = self.alive_other_members();
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if !alive.is_empty() {
                    let pos = alive.iter().position(|i| *i == prompt.selected).unwrap_or(0);
                    let new_pos = (pos + alive.len() - 1) % alive.len();
                    self.switch_prompt = Some(SwitchPromptState {
                        selected: alive[new_pos],
                        forced: prompt.forced,
                    });
                }
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !alive.is_empty() {
                    let pos = alive.iter().position(|i| *i == prompt.selected).unwrap_or(0);
                    let new_pos = (pos + 1) % alive.len();
                    self.switch_prompt = Some(SwitchPromptState {
                        selected: alive[new_pos],
                        forced: prompt.forced,
                    });
                }
                Transition::Stay
            }
            KeyCode::Enter => {
                let was_forced = prompt.forced;
                self.switch_prompt = None;
                self.perform_switch(prompt.selected, player);
                if was_forced {
                    // Forced (faint) swap: the enemy's turn was already
                    // queued or running; just resume the round logic. Do
                    // NOT cost a turn — the player didn't choose to faint.
                } else {
                    // Voluntary switch costs the player's action this round.
                    let mut rng = rand::thread_rng();
                    self.fight
                        .set_message("You swapped in a fresh teammate!", 0.5);
                    self.round_active = true;
                    if let Some(enemy_attack) = self.pick_enemy_move(&mut rng) {
                        self.next_action = Some(QueuedAction {
                            actor: Actor::Enemy,
                            attack: enemy_attack,
                        });
                    }
                }
                Transition::Stay
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                if !prompt.forced {
                    self.switch_prompt = None;
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    /// Player committed to an attack from the menu. Validate mana, pick
    /// the enemy's response, roll speed-based order, then start the
    /// faster combatant's animation. The slower one becomes `next_action`.
    fn commit_player_attack(&mut self) {
        let idx = self.fight.attack_selected;
        if idx >= self.fight.attacks.len() {
            return;
        }
        let attack = self.fight.attacks[idx].clone();
        if self.fight.player_mana < attack.mana_cost {
            self.fight.set_message("Not enough mana!", 0.8);
            return;
        }
        self.fight.player_mana -= attack.mana_cost;
        self.fight.menu_state = MenuState::Main;
        self.round_active = true;

        let mut rng = rand::thread_rng();
        let enemy_choice = self.pick_enemy_move(&mut rng);
        self.fight.roll_round_order(&mut rng);

        let player_action = QueuedAction {
            actor: Actor::Player,
            attack,
        };

        match enemy_choice {
            Some(enemy_attack) => {
                let enemy_action = QueuedAction {
                    actor: Actor::Enemy,
                    attack: enemy_attack,
                };
                if self.fight.enemy_first_this_round {
                    self.next_action = Some(player_action);
                    self.start_action(enemy_action);
                } else {
                    self.next_action = Some(enemy_action);
                    self.start_action(player_action);
                }
            }
            None => {
                // Enemy has no usable move — player just acts.
                self.start_action(player_action);
            }
        }
    }

    /// Pick a uniform-random move from the enemy's moveset, resolved
    /// against the global attack registry.
    fn pick_enemy_move<R: rand::Rng>(&self, rng: &mut R) -> Option<crate::fight::Attack> {
        let name = self.fight.enemy.moveset.choose(rng)?;
        attack_lib::find_by_name(name)
    }

    /// Start an action's animation and stash the chosen attack so the
    /// "anim done" branch can apply its effect.
    fn start_action(&mut self, action: QueuedAction) {
        let QueuedAction { actor, attack } = action;
        match actor {
            Actor::Player => {
                let start_x = self.crab.position.0;
                let target_x = (self.last_terminal_size.0 as f32 - 18.0).max(start_x + 5.0);
                self.fight
                    .set_message(format!("You used {}!", attack.name), 0.6);
                self.fight.animation = Some(Animation::for_attack(&attack, start_x, target_x));
                self.fight.pending_player_attack = Some(attack);
            }
            Actor::Enemy => {
                let enemy_name = self.fight.enemy.name.clone();
                self.fight
                    .set_message(format!("{} used {}!", enemy_name, attack.name), 0.7);
                let (start_x, _) = self.enemy_base_position();
                let target_x = self.crab.position.0;
                self.fight.animation = Some(Animation::for_enemy_attack(&attack, start_x, target_x));
                self.fight.pending_enemy_attack = Some(attack);
            }
        }
    }

    /// Where the enemy sprite is anchored in the scene (top-left corner).
    /// Mirrors what `widgets::render_enemy` uses by default.
    fn enemy_base_position(&self) -> (f32, f32) {
        let scene_w = self.last_terminal_size.0 as f32;
        let scene_h = self.last_terminal_size.1 as f32;
        let sprite_w = self
            .fight
            .enemy
            .sprite
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as f32;
        let sprite_h = self.fight.enemy.sprite.len() as f32;
        let x = (scene_w - sprite_w - 4.0).max(0.0);
        let y = (scene_h - sprite_h - 1.0).max(0.0);
        (x, y)
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
            KeyCode::Enter => {
                self.use_focused_item();
            }
            _ => {}
        }
        Transition::Stay
    }

    /// Apply the selected inventory item in combat. Only HP and Mana
    /// potions resolve here for now — anything else flashes a "can't use
    /// here" message and doesn't consume a turn.
    fn use_focused_item(&mut self) {
        let idx = self.fight.item_selected;
        let Some(stack) = self.fight.items.get(idx) else {
            return;
        };
        let item = stack.item.clone();
        let result = match &item {
            Item::HpPotion(size) => {
                let amount = match size {
                    PotionSize::Small => 10,
                    PotionSize::Large => 30,
                };
                let before = self.fight.player_hp;
                self.fight.player_hp = (self.fight.player_hp + amount).min(self.fight.player_max_hp);
                let healed = self.fight.player_hp.saturating_sub(before);
                Some(format!("Recovered {} HP.", healed))
            }
            Item::ManaPotion(size) => {
                let amount = match size {
                    PotionSize::Small => 6,
                    PotionSize::Large => 15,
                };
                self.fight.player_mana = (self.fight.player_mana + amount)
                    .min(self.fight.player_max_mana);
                Some(format!("Recovered {} MP.", amount))
            }
            _ => None,
        };

        let Some(message) = result else {
            self.fight.set_message("Can't use that here.", 0.8);
            return;
        };

        // Consume one from the stack and prune empties.
        if let Some(stack) = self.fight.items.get_mut(idx) {
            stack.count = stack.count.saturating_sub(1);
        }
        if let Some(stack) = self.fight.items.get(idx) {
            if stack.count == 0 {
                self.fight.items.remove(idx);
                if self.fight.item_selected >= self.fight.items.len()
                    && !self.fight.items.is_empty()
                {
                    self.fight.item_selected = self.fight.items.len() - 1;
                }
            }
        }
        self.fight.set_message(message, 1.0);
        self.fight.menu_state = MenuState::Main;
        self.round_active = true;
        // Using an item costs the player's turn this round. Queue the
        // enemy's reaction to fire after the item message clears.
        let mut rng = rand::thread_rng();
        if let Some(enemy_attack) = self.pick_enemy_move(&mut rng) {
            self.next_action = Some(QueuedAction {
                actor: Actor::Enemy,
                attack: enemy_attack,
            });
        }
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
                    let hp_before = self.fight.player_hp;
                    let damage = self.fight.resolve_player_attack(&attack, &mut rng);
                    if matches!(attack.effect, crate::fight::Effect::Damage(_)) && damage > 0 {
                        self.fight.flash_enemy();
                    }
                    let enemy_name = self.fight.enemy.name.clone();
                    let msg = match attack.effect {
                        crate::fight::Effect::Damage(_) => {
                            if self.fight.enemy.hp == 0 {
                                format!("{} fainted!", enemy_name)
                            } else if damage > 0 {
                                format!("{} took {} damage!", enemy_name, damage)
                            } else {
                                format!("It had no effect on {}.", enemy_name)
                            }
                        }
                        crate::fight::Effect::Heal(_) => {
                            let healed = self.fight.player_hp.saturating_sub(hp_before);
                            format!("Recovered {} HP.", healed)
                        }
                        crate::fight::Effect::Buff { kind, magnitude, .. } => {
                            format!("Your {} rose by {}%!", kind.label(), magnitude)
                        }
                    };
                    self.fight.set_message(msg, 1.0);
                    if self.fight.enemy.hp == 0 {
                        self.pending_exit = Some(FightOutcome::Victory);
                    }
                } else if let Some(attack) = self.fight.pending_enemy_attack.take() {
                    let mut rng = rand::thread_rng();
                    let damage = self.fight.resolve_enemy_attack(&attack, &mut rng);
                    if matches!(attack.effect, crate::fight::Effect::Damage(_)) && damage > 0 {
                        self.fight.flash_player();
                    }
                    let enemy_name = self.fight.enemy.name.clone();
                    let msg = match attack.effect {
                        crate::fight::Effect::Damage(_) => {
                            if self.fight.player_hp == 0 {
                                "You fainted!".to_string()
                            } else if damage > 0 {
                                format!("You took {} damage!", damage)
                            } else {
                                "It had no effect.".to_string()
                            }
                        }
                        crate::fight::Effect::Heal(_) => {
                            format!("{} regained some HP.", enemy_name)
                        }
                        crate::fight::Effect::Buff { kind, magnitude, .. } => {
                            format!("{}'s {} rose by {}%!", enemy_name, kind.label(), magnitude)
                        }
                    };
                    self.fight.set_message(msg, 1.0);
                    if self.fight.player_hp == 0 {
                        // Sync the 0 HP into the active party member now
                        // so alive_other_members has accurate state.
                        self.commit_active_member(player);
                        if has_revive_pearl(&self.fight.items) {
                            self.revive_prompt_pending = true;
                        } else if !self.alive_other_members().is_empty() {
                            // Other party members can step in.
                            self.faint_swap_pending = true;
                        } else {
                            self.pending_exit = Some(FightOutcome::Defeat);
                        }
                    }
                }
            }
        } else if bounds.0 > 0.0 && bounds.1 > 0.0 {
            self.crab.walk_range_x = Some((0.0, bounds.0 * 0.4));
            self.crab.update(dt, bounds);
        }

        self.environment.update_cycle(dt, 1.0, 1.0);
        self.fight.tick_message(dt);
        self.fight.tick_hit_flashes(dt);

        // Once the impact message clears, surface the deferred Revive
        // Pearl prompt rather than continuing into round logic.
        if self.revive_prompt_pending
            && self.fight.animation.is_none()
            && self.fight.message.is_none()
        {
            self.revive_prompt_pending = false;
            self.revive_prompt_open = true;
        }

        // Same gate for the forced member-select on faint.
        if self.faint_swap_pending
            && self.fight.animation.is_none()
            && self.fight.message.is_none()
        {
            self.faint_swap_pending = false;
            self.switch_prompt = Some(SwitchPromptState {
                selected: self.first_alive_other(),
                forced: true,
            });
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
            if let Some(action) = self.next_action.take() {
                // Second action of the current round resolves now.
                self.start_action(action);
            } else if self.round_active {
                // Round complete; tick buffs once and wait for the next
                // player input. Order is rerolled inside
                // commit_player_attack when the player actually acts.
                self.fight.tick_buffs();
                self.round_active = false;
            }
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
            FightOutcome::Flee => {
                self.commit_active_member(player);
                self.exit_fight()
            }
            FightOutcome::Defeat => self.defeat(player),
        }
    }

    fn defeat(&mut self, player: &Player) -> Transition {
        // Sync the active member's 0 HP to the party before we drop the map.
        self.commit_active_member(player);
        let Some(map) = self.map.take() else {
            return self.exit_fight();
        };
        let starter = map.run.active_member().template.clone();
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
        // Persist the active member's post-fight HP/MP into the party
        // before the map moves on.
        self.commit_active_member(player);
        let (Some(map), Some(kind)) = (self.map.take(), self.node_kind) else {
            // Standalone fight without a run — nothing to reward.
            return self.exit_fight();
        };
        let mut rng = rand::thread_rng();
        let (gold, items) = roll_rewards(kind, player, &mut rng);
        apply_rewards(player, gold, &items);

        // Cross-run embers: 1 per fight cleared, +10 bonus on boss kill.
        let ember_drip = 1;
        let ember_bonus = if matches!(kind, NodeKind::Boss) { 10 } else { 0 };
        let embers_earned = ember_drip + ember_bonus;
        crate::meta::add_embers(embers_earned);

        // Boss kills can drop unowned starters into the post-run shop.
        if matches!(kind, NodeKind::Boss) {
            roll_starter_recruits(&mut rng);
        }

        if matches!(kind, NodeKind::Boss) {
            let starter = map.run.active_member().template.clone();
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
                embers_earned,
                items,
            )));
        }

        // Detour into the capture prompt if the player has a Monster Net
        // and the kind is catchable. Otherwise go straight to RewardScreen.
        let enemy_species = self.fight.enemy.name.clone();
        let catch_rate = crate::ui::screens::capture::catch_rate(kind);
        let has_net = crate::ui::screens::capture::has_net(&player.inventory);
        if let (Some(rate), true) = (catch_rate, has_net) {
            return Transition::Goto(Screen::CapturePrompt(
                crate::ui::screens::CapturePromptScreen::new(
                    map,
                    gold,
                    embers_earned,
                    items,
                    kind,
                    enemy_species,
                    rate,
                ),
            ));
        }

        Transition::Goto(Screen::Reward(RewardScreen::new(
            map,
            gold,
            embers_earned,
            items,
            kind,
        )))
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

        // The animation displaces whichever combatant is the attacker.
        let enemy_base = self.enemy_base_position();
        let (crab_override, enemy_override) = match self.fight.animation.as_ref() {
            Some(anim) => match anim.side {
                crate::fight::AttackerSide::Player => {
                    (Some(anim.crab_position(self.crab.position)), None)
                }
                crate::fight::AttackerSide::Enemy => {
                    (None, Some(anim.crab_position(enemy_base)))
                }
            },
            None => (None, None),
        };

        widgets::render_environment_background(frame, &self.environment, scene_area);
        if self.fight.player_visible() {
            widgets::render_crab(frame, &self.crab, scene_area, crab_override);
        }
        if self.fight.enemy_visible() {
            widgets::render_enemy(frame, &self.fight.enemy, scene_area, enemy_override);
        }
        if let Some(anim) = self.fight.animation.as_ref() {
            // Particles and projectile arc use whoever the attacker is as
            // their reference base.
            let attacker_base = match anim.side {
                crate::fight::AttackerSide::Player => self.crab.position,
                crate::fight::AttackerSide::Enemy => enemy_base,
            };
            widgets::render_projectile(frame, anim, attacker_base.1, scene_area);
            widgets::render_particles(frame, anim, attacker_base, scene_area);
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

        if self.revive_prompt_open {
            render_revive_prompt(frame, area);
        }
        if let Some(prompt) = self.switch_prompt.as_ref() {
            if let Some(map) = self.map.as_ref() {
                render_switch_prompt(frame, area, &map.run, prompt);
            }
        }
    }
}

/// 33% chance per unowned starter to drop into the post-run shop. Run
/// once on boss kill.
fn roll_starter_recruits<R: rand::Rng>(rng: &mut R) {
    use crate::data::starters;
    use crate::meta;
    let snap = meta::snapshot();
    for starter in starters::all_starters() {
        let id = meta::starter_id(&starter.name);
        let already_owned = snap.monsters.contains_key(&id);
        let already_pending = snap
            .pending_captures
            .iter()
            .any(|c| c.id == id);
        if already_owned || already_pending {
            continue;
        }
        if rng.gen_bool(0.33) {
            meta::push_pending_capture(meta::MonsterInstance {
                id,
                species: starter.name,
            });
        }
    }
}

fn has_revive_pearl(items: &[crate::fight::ItemStack]) -> bool {
    use crate::fight::{Item, UtilityKind};
    items
        .iter()
        .any(|s| matches!(&s.item, Item::Utility(UtilityKind::Revive)) && s.count > 0)
}

fn consume_one_revive_pearl(items: &mut Vec<crate::fight::ItemStack>) {
    use crate::fight::{Item, UtilityKind};
    if let Some(idx) = items
        .iter()
        .position(|s| matches!(&s.item, Item::Utility(UtilityKind::Revive)) && s.count > 0)
    {
        items[idx].count = items[idx].count.saturating_sub(1);
        if items[idx].count == 0 {
            items.remove(idx);
        }
    }
}

fn render_switch_prompt(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    run: &crate::run::Run,
    prompt: &SwitchPromptState,
) {
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let popup_w: u16 = 50.min(area.width.saturating_sub(4)).max(20);
    let popup_h: u16 =
        (run.party.len() as u16 + 5).min(area.height.saturating_sub(2)).max(7);
    if popup_w < 20 || popup_h < 5 {
        return;
    }
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    // Dim outside, clear inside.
    let buf = frame.buffer_mut();
    for y in area.y..(area.y + area.height) {
        for x in area.x..(area.x + area.width) {
            if x >= popup.x
                && x < popup.x + popup.width
                && y >= popup.y
                && y < popup.y + popup.height
            {
                continue;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                let fg = match cell.fg {
                    Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                    other => other,
                };
                cell.set_fg(fg);
            }
        }
    }
    for y in popup.y..(popup.y + popup.height) {
        for x in popup.x..(popup.x + popup.width) {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_style(Style::default());
            }
        }
    }

    let title = if prompt.forced {
        " Choose a teammate "
    } else {
        " Switch member "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(160, 220, 200)))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Rgb(160, 220, 200))
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    if inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    let headline = if prompt.forced {
        format!(
            "{} fainted. Send in...",
            run.party
                .get(run.active)
                .map(|m| m.template.name.as_str())
                .unwrap_or("Your monster")
        )
    } else {
        "Swap to which teammate? (costs your turn)".to_string()
    };
    lines.push(Line::from(Span::styled(
        headline,
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    for (idx, member) in run.party.iter().enumerate() {
        let is_active = idx == run.active;
        let is_alive = member.current_hp > 0;
        let is_selected = idx == prompt.selected;
        let style = if is_active || !is_alive {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let cursor = if is_selected && !is_active && is_alive {
            "\u{25B6} "
        } else {
            "  "
        };
        let suffix = if is_active {
            " (active)"
        } else if !is_alive {
            " (fainted)"
        } else {
            ""
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(
                format!(
                    "{}  HP {}/{}{}",
                    member.template.name, member.current_hp, member.max_hp, suffix
                ),
                style,
            ),
        ]));
    }
    lines.push(Line::from(""));
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint_spans = if prompt.forced {
        vec![
            Span::styled("\u{2191}\u{2193}", key),
            Span::styled(" pick   ", dim),
            Span::styled("Enter", key),
            Span::styled(" send", dim),
        ]
    } else {
        vec![
            Span::styled("\u{2191}\u{2193}", key),
            Span::styled(" pick   ", dim),
            Span::styled("Enter", key),
            Span::styled(" swap   ", dim),
            Span::styled("Esc", key),
            Span::styled(" cancel", dim),
        ]
    };
    lines.push(Line::from(hint_spans));
    frame.render_widget(
        Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center),
        inner,
    );
}

fn render_revive_prompt(frame: &mut Frame, area: ratatui::layout::Rect) {
    use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let popup_w: u16 = 50.min(area.width.saturating_sub(4)).max(20);
    let popup_h: u16 = 8.min(area.height.saturating_sub(2)).max(5);
    if popup_w < 20 || popup_h < 5 {
        return;
    }
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    // Dim everything else, then clear the popup background.
    let buf = frame.buffer_mut();
    for y in area.y..(area.y + area.height) {
        for x in area.x..(area.x + area.width) {
            if x >= popup.x
                && x < popup.x + popup.width
                && y >= popup.y
                && y < popup.y + popup.height
            {
                continue;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                let fg = match cell.fg {
                    Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                    other => other,
                };
                cell.set_fg(fg);
            }
        }
    }
    for y in popup.y..(popup.y + popup.height) {
        for x in popup.x..(popup.x + popup.width) {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_style(Style::default());
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(220, 180, 255)))
        .title(Span::styled(
            " Revive Pearl ",
            Style::default()
                .fg(Color::Rgb(220, 180, 255))
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    if inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "You fell. Burn a Revive Pearl?",
            Style::default().fg(Color::Gray),
        )))
        .alignment(Alignment::Center),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "(restores HP and MP to pre-fight values)",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center),
        chunks[1],
    );

    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let prompt = Line::from(vec![
        Span::styled("Y", key),
        Span::styled(" / ", dim),
        Span::styled("Enter", key),
        Span::styled("  consume   ", dim),
        Span::styled("N", key),
        Span::styled(" / ", dim),
        Span::styled("Esc", key),
        Span::styled("  game over", dim),
    ]);
    frame.render_widget(
        Paragraph::new(prompt).alignment(Alignment::Center),
        chunks[2],
    );
}
