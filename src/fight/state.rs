use super::actions::Action;
use super::enemy::Enemy;

pub struct FightState {
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub enemy: Enemy,
    pub floor: u32,
    pub selected_action: usize,
}

impl FightState {
    pub fn new() -> Self {
        Self {
            player_hp: 50,
            player_max_hp: 50,
            enemy: Enemy::slime(),
            floor: 1,
            selected_action: 0,
        }
    }

    pub fn selected(&self) -> Action {
        Action::ALL[self.selected_action]
    }
}
