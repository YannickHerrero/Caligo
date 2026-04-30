use super::actions::Action;
use super::animation::Animation;
use super::attack::{Attack, Effect};
use super::enemy::Enemy;
use super::item::ItemStack;
use crate::data::enemies;
use crate::player::Player;

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
        }
    }

    /// Apply a finished player attack's effect against the enemy. Returns
    /// the raw damage dealt (0 for non-damage effects).
    pub fn resolve_player_attack(&mut self, attack: &Attack) -> u32 {
        match attack.effect {
            Effect::Damage(base) => {
                let mult = attack
                    .element
                    .effectiveness_vs(self.enemy.primary_type, self.enemy.secondary_type);
                let damage = ((base as f32) * mult).round().max(1.0) as u32;
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

    pub fn commit_to_player(&self, player: &mut Player) {
        player.hp = self.player_hp.min(player.max_hp());
        player.mana = self.player_mana.min(player.max_mana());
        player.inventory = self.items.clone();
    }

    pub fn selected(&self) -> Action {
        Action::ALL[self.selected_action]
    }
}
