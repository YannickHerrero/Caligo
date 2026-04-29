use super::actions::Action;
use super::attack::Attack;
use super::enemy::Enemy;
use super::item::Item;

pub struct FightState {
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub enemy: Enemy,
    pub floor: u32,
    pub selected_action: usize,
    pub attacks: Vec<Attack>,
    pub items: Vec<Item>,
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
                Attack::new("Pinch"),
                Attack::new("Bubble"),
                Attack::new("Snip"),
                Attack::new("Shell Bash"),
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
        }
    }

    pub fn selected(&self) -> Action {
        Action::ALL[self.selected_action]
    }
}
