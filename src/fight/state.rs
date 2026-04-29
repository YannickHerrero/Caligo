use super::actions::Action;
use super::animation::Animation;
use super::attack::{AnimationKind, Attack};
use super::enemy::Enemy;
use super::item::Item;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    Main,
    AttackSelect,
    ItemSelect,
}

pub struct FightState {
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub enemy: Enemy,
    pub floor: u32,
    pub selected_action: usize,
    pub attacks: Vec<Attack>,
    pub items: Vec<Item>,
    pub menu_state: MenuState,
    pub attack_selected: usize,
    pub item_selected: usize,
    pub item_scroll: usize,
    pub animation: Option<Animation>,
}

impl FightState {
    pub fn new() -> Self {
        Self {
            player_hp: 50,
            player_max_hp: 50,
            enemy: Enemy::slime(),
            floor: 1,
            selected_action: 0,
            attacks: vec![
                Attack::new("Pinch", AnimationKind::Dash),
                Attack::new("Bubble", AnimationKind::EnergyBall),
                Attack::new("Snip", AnimationKind::Jump),
                Attack::new("Shell Bash", AnimationKind::Dash),
            ],
            items: vec![
                Item::new("Small Potion"),
                Item::new("Medium Potion"),
                Item::new("Large Potion"),
                Item::new("Antidote"),
                Item::new("Smoke Bomb"),
                Item::new("Throwing Knife"),
                Item::new("Pearl"),
                Item::new("Lucky Shell"),
                Item::new("Sea Salt"),
                Item::new("Kelp Wrap"),
                Item::new("Driftwood"),
                Item::new("Sand Dollar"),
            ],
            menu_state: MenuState::Main,
            attack_selected: 0,
            item_selected: 0,
            item_scroll: 0,
            animation: None,
        }
    }

    pub fn selected(&self) -> Action {
        Action::ALL[self.selected_action]
    }
}
