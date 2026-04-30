use super::actions::Action;
use super::animation::Animation;
use super::attack::Attack;
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
