use super::actions::Action;
use super::animation::Animation;
use super::attack::{AnimationKind, Attack, Element};
use super::enemy::Enemy;
use super::item::{Item, ItemStack, PotionSize, TrinketKind, UtilityKind};
use super::projectile::ProjectileKind;

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
    pub items: Vec<ItemStack>,
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
                Attack::new(
                    "Pinch",
                    AnimationKind::Dash,
                    5,
                    0,
                    Element::Neutral,
                    "A quick claw pinch. No mana cost, modest damage.",
                ),
                Attack::new(
                    "Bubble",
                    AnimationKind::Throw(ProjectileKind::Water),
                    7,
                    3,
                    Element::Water,
                    "Lobs a bubble that splashes the enemy.",
                ),
                Attack::new(
                    "Snip",
                    AnimationKind::Jump,
                    8,
                    2,
                    Element::Neutral,
                    "Leaping snip with both claws.",
                ),
                Attack::new(
                    "Cosmic Orb",
                    AnimationKind::Throw(ProjectileKind::EnergyBall),
                    14,
                    8,
                    Element::Air,
                    "A heavy orb of cosmic energy. High cost, high damage.",
                ),
            ],
            items: vec![
                ItemStack::new(Item::HpPotion(PotionSize::Small), 3),
                ItemStack::new(Item::HpPotion(PotionSize::Large), 1),
                ItemStack::new(Item::ManaPotion(PotionSize::Small), 2),
                ItemStack::new(Item::ManaPotion(PotionSize::Large), 1),
                ItemStack::new(
                    Item::AttackStone {
                        attack_name: "Tide Slam".to_string(),
                    },
                    1,
                ),
                ItemStack::new(Item::Trinket(TrinketKind::HeartCharm), 1),
                ItemStack::new(Item::Trinket(TrinketKind::ManaPearl), 1),
                ItemStack::new(Item::Trinket(TrinketKind::LuckyShell), 1),
                ItemStack::new(Item::Utility(UtilityKind::Revive), 1),
                ItemStack::new(Item::Utility(UtilityKind::EscapeToken), 2),
                ItemStack::new(Item::Utility(UtilityKind::GoldPouch), 1),
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
