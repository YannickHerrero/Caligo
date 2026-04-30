use super::actions::Action;
use super::animation::Animation;
use super::attack::{Attack, Effect, Element};
use super::enemy::Enemy;
use super::item::ItemStack;
use crate::data::enemies;
use crate::player::Player;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    Main,
    AttackSelect,
    ItemSelect,
}

pub struct FightState {
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub player_mana: u32,
    pub enemy: Enemy,
    pub floor: u32,
    pub selected_action: usize,
    pub attacks: Vec<Attack>,
    pub items: Vec<ItemStack>,
    pub menu_state: MenuState,
    pub attack_selected: usize,
    pub item_selected: usize,
    pub item_scroll: usize,
    pub animation: Option<Animation>,
    /// The attack that's currently animating from the player. When the
    /// animation finishes, its effect is applied to the enemy.
    pub pending_player_attack: Option<Attack>,
    /// One-line action feedback shown in place of the action menu while a
    /// turn is resolving. Cleared once `message_linger` ticks down to 0
    /// and there's no animation in flight.
    pub message: Option<String>,
    pub message_linger: f32,
    /// Set after the player's turn resolves to telegraph the enemy's
    /// counter-strike: holds the attack the enemy is about to use. While
    /// this is `Some`, the screen is showing "X used Y!" and waiting for
    /// the message to linger before applying damage to the player.
    pub pending_enemy_attack: Option<Attack>,
    /// Player's elemental type, derived from the chosen starter. Used
    /// when computing how effective enemy moves are against us. `None`
    /// for stand-alone debug fights.
    pub player_type: Option<Element>,
}

impl FightState {
    pub fn from_player(player: &Player) -> Self {
        Self::from_player_with_enemy(player, enemies::slime())
    }

    pub fn from_player_with_enemy(player: &Player, enemy: Enemy) -> Self {
        let attacks: Vec<Attack> = player
            .equipped_attacks_resolved()
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        Self {
            player_hp: player.hp,
            player_max_hp: player.max_hp(),
            player_mana: player.mana,
            enemy,
            floor: 1,
            selected_action: 0,
            attacks,
            items: player.inventory.clone(),
            menu_state: MenuState::Main,
            attack_selected: 0,
            item_selected: 0,
            item_scroll: 0,
            animation: None,
            pending_player_attack: None,
            message: None,
            message_linger: 0.0,
            pending_enemy_attack: None,
            player_type: None,
        }
    }

    pub fn set_message(&mut self, text: impl Into<String>, linger: f32) {
        self.message = Some(text.into());
        self.message_linger = linger;
    }

    /// Tick the message timer and clear once it elapses (and no animation
    /// is mid-flight).
    pub fn tick_message(&mut self, dt: f32) {
        if self.message.is_none() {
            return;
        }
        if self.animation.is_some() {
            // Keep the message pinned while the animation plays.
            return;
        }
        self.message_linger -= dt;
        if self.message_linger <= 0.0 {
            self.message = None;
            self.message_linger = 0.0;
        }
    }

    /// Apply a finished player attack's effect against the enemy. Returns
    /// the raw damage dealt (0 for non-damage effects).
    pub fn resolve_player_attack<R: Rng>(&mut self, attack: &Attack, rng: &mut R) -> u32 {
        match attack.effect {
            Effect::Damage(base) => {
                let mult = attack
                    .element
                    .effectiveness_vs(self.enemy.primary_type, self.enemy.secondary_type);
                let variance = rng.gen_range(0.9..=1.1);
                let damage = ((base as f32) * mult * variance).round().max(1.0) as u32;
                self.enemy.hp = self.enemy.hp.saturating_sub(damage);
                damage
            }
            Effect::Heal(amount) => {
                self.player_hp = (self.player_hp + amount).min(self.player_max_hp);
                0
            }
            Effect::Buff { .. } => 0,
        }
    }

    /// Apply a finished enemy attack's effect against the player. Type
    /// effectiveness is computed against the player's starter type when
    /// known.
    pub fn resolve_enemy_attack<R: Rng>(&mut self, attack: &Attack, rng: &mut R) -> u32 {
        match attack.effect {
            Effect::Damage(base) => {
                let mult = match self.player_type {
                    Some(t) => attack.element.effectiveness_vs(t, None),
                    None => 1.0,
                };
                let variance = rng.gen_range(0.9..=1.1);
                let damage = ((base as f32) * mult * variance).round().max(1.0) as u32;
                self.player_hp = self.player_hp.saturating_sub(damage);
                damage
            }
            Effect::Heal(amount) => {
                self.enemy.hp = (self.enemy.hp + amount).min(self.enemy.max_hp);
                0
            }
            Effect::Buff { .. } => 0,
        }
    }

    pub fn commit_to_player(&self, player: &mut Player) {
        player.hp = self.player_hp.min(player.max_hp());
        player.mana = self.player_mana.min(player.max_mana());
        player.inventory = self.items.clone();
    }

    pub fn selected(&self) -> Action {
        Action::ALL[self.selected_action]
    }
}
