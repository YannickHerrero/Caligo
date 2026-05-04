use super::actions::Action;
use super::animation::Animation;
use super::attack::{Attack, BuffKind, Effect, Element};
use super::enemy::Enemy;
use super::item::ItemStack;
use crate::data::enemies;
use crate::player::Player;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct ActiveBuff {
    pub kind: BuffKind,
    pub magnitude: u32,
    pub turns_remaining: u32,
}

impl ActiveBuff {
    /// Returns the additive percentage represented by this buff (e.g.
    /// magnitude 25 → 0.25). Sign is positive for both ATK-up and DEF-up;
    /// callers compose them based on context.
    pub fn pct(&self) -> f32 {
        self.magnitude as f32 / 100.0
    }
}

/// Sum the AttackUp percentage across an active-buff list.
pub fn sum_attack_pct(buffs: &[ActiveBuff]) -> f32 {
    buffs
        .iter()
        .filter(|b| b.kind == BuffKind::AttackUp)
        .map(|b| b.pct())
        .sum()
}

/// Sum the DefenseUp percentage across an active-buff list.
pub fn sum_defense_pct(buffs: &[ActiveBuff]) -> f32 {
    buffs
        .iter()
        .filter(|b| b.kind == BuffKind::DefenseUp)
        .map(|b| b.pct())
        .sum()
}

fn tick_buff_list(buffs: &mut Vec<ActiveBuff>) {
    for b in buffs.iter_mut() {
        b.turns_remaining = b.turns_remaining.saturating_sub(1);
    }
    buffs.retain(|b| b.turns_remaining > 0);
}

/// How long a hit-flash lasts after damage applies.
pub const HIT_FLASH_DURATION: f32 = 0.4;
/// Half-cycle of the blink — the sprite is hidden for one period and
/// shown for the next.
const HIT_FLASH_PERIOD: f32 = 0.07;

fn sprite_visible(remaining: f32) -> bool {
    if remaining <= 0.0 {
        return true;
    }
    let elapsed = HIT_FLASH_DURATION - remaining;
    let cycle = (elapsed / HIT_FLASH_PERIOD).floor() as i32;
    // Even cycle -> hidden, odd cycle -> shown. Starting hidden makes the
    // first frame after impact already register as a hit.
    cycle % 2 != 0
}

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
    pub player_max_mana: u32,
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
    /// Player's speed at the start of the fight. Used to roll round
    /// order against `enemy.speed`.
    pub player_speed: u32,
    /// Whose turn comes first in the current round, rolled at round
    /// start (random tie-break when speeds match).
    pub enemy_first_this_round: bool,
    /// Active buffs on the player (self-targeted from moves like Sharpen
    /// and Carapace). Apply during damage calc, tick down at round end.
    pub player_buffs: Vec<ActiveBuff>,
    /// Active buffs on the enemy.
    pub enemy_buffs: Vec<ActiveBuff>,
    /// Seconds of hit-flash remaining on the player. Drives a blink in
    /// `render_crab` until it ticks down to 0.
    pub player_hit_remaining: f32,
    /// Seconds of hit-flash remaining on the enemy.
    pub enemy_hit_remaining: f32,
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
            player_max_mana: player.max_mana(),
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
            player_speed: player.speed,
            enemy_first_this_round: false,
            player_buffs: Vec::new(),
            enemy_buffs: Vec::new(),
            player_hit_remaining: 0.0,
            enemy_hit_remaining: 0.0,
        }
    }

    /// Roll initiative for a new round. Faster combatant goes first;
    /// ties roll a coin.
    pub fn roll_round_order<R: Rng>(&mut self, rng: &mut R) {
        self.enemy_first_this_round = match self.enemy.speed.cmp(&self.player_speed) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => rng.gen_bool(0.5),
        };
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
                let type_mult = attack
                    .element
                    .effectiveness_vs(self.enemy.primary_type, self.enemy.secondary_type);
                let atk_pct = sum_attack_pct(&self.player_buffs);
                let def_pct = sum_defense_pct(&self.enemy_buffs);
                let variance = rng.gen_range(0.9..=1.1);
                let damage = ((base as f32)
                    * type_mult
                    * (1.0 + atk_pct)
                    * (1.0 - def_pct).max(0.0)
                    * variance)
                    .round()
                    .max(1.0) as u32;
                self.enemy.hp = self.enemy.hp.saturating_sub(damage);
                damage
            }
            Effect::Heal(amount) => {
                self.player_hp = (self.player_hp + amount).min(self.player_max_hp);
                0
            }
            Effect::Buff {
                kind,
                magnitude,
                duration,
            } => {
                self.player_buffs.push(ActiveBuff {
                    kind,
                    magnitude,
                    turns_remaining: duration,
                });
                0
            }
        }
    }

    /// Apply a finished enemy attack's effect against the player. Type
    /// effectiveness is computed against the player's starter type when
    /// known.
    pub fn resolve_enemy_attack<R: Rng>(&mut self, attack: &Attack, rng: &mut R) -> u32 {
        match attack.effect {
            Effect::Damage(base) => {
                let type_mult = match self.player_type {
                    Some(t) => attack.element.effectiveness_vs(t, None),
                    None => 1.0,
                };
                let atk_pct = sum_attack_pct(&self.enemy_buffs);
                let def_pct = sum_defense_pct(&self.player_buffs);
                let variance = rng.gen_range(0.9..=1.1);
                let damage = ((base as f32)
                    * type_mult
                    * (1.0 + atk_pct)
                    * (1.0 - def_pct).max(0.0)
                    * variance)
                    .round()
                    .max(1.0) as u32;
                self.player_hp = self.player_hp.saturating_sub(damage);
                damage
            }
            Effect::Heal(amount) => {
                self.enemy.hp = (self.enemy.hp + amount).min(self.enemy.max_hp);
                0
            }
            Effect::Buff {
                kind,
                magnitude,
                duration,
            } => {
                self.enemy_buffs.push(ActiveBuff {
                    kind,
                    magnitude,
                    turns_remaining: duration,
                });
                0
            }
        }
    }

    /// Decrement remaining turns on every active buff and prune expired
    /// ones. Call at the end of each round.
    pub fn tick_buffs(&mut self) {
        tick_buff_list(&mut self.player_buffs);
        tick_buff_list(&mut self.enemy_buffs);
    }

    /// Start a fresh hit-flash on the player.
    pub fn flash_player(&mut self) {
        self.player_hit_remaining = HIT_FLASH_DURATION;
    }

    /// Start a fresh hit-flash on the enemy.
    pub fn flash_enemy(&mut self) {
        self.enemy_hit_remaining = HIT_FLASH_DURATION;
    }

    /// Tick both hit-flash timers.
    pub fn tick_hit_flashes(&mut self, dt: f32) {
        self.player_hit_remaining = (self.player_hit_remaining - dt).max(0.0);
        self.enemy_hit_remaining = (self.enemy_hit_remaining - dt).max(0.0);
    }

    /// Should the player sprite be drawn this frame, accounting for the
    /// blink while hit-flashing?
    pub fn player_visible(&self) -> bool {
        sprite_visible(self.player_hit_remaining)
    }

    /// Should the enemy sprite be drawn this frame?
    pub fn enemy_visible(&self) -> bool {
        sprite_visible(self.enemy_hit_remaining)
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
