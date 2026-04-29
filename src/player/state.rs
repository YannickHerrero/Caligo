use crate::fight::{
    AnimationKind, Attack, Element, Item, ItemStack, PotionSize, ProjectileKind, TrinketKind,
    MAX_ATTACKS,
};

pub const MAX_TRINKETS: usize = 2;

pub type EquippedTrinkets = [Option<TrinketKind>; MAX_TRINKETS];

pub struct Player {
    pub hp: u32,
    pub base_max_hp: u32,
    pub mana: u32,
    pub base_max_mana: u32,
    pub gold: u32,
    pub owned_attacks: Vec<Attack>,
    pub equipped_attacks: [Option<usize>; MAX_ATTACKS],
    pub inventory: Vec<ItemStack>,
    pub equipped_trinkets: EquippedTrinkets,
}

impl Player {
    pub fn new() -> Self {
        let owned_attacks = vec![
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
        ];
        let equipped_attacks = [Some(0), Some(1), Some(2), Some(3)];

        let inventory = vec![ItemStack::new(Item::HpPotion(PotionSize::Small), 2)];

        Self {
            hp: 25,
            base_max_hp: 25,
            mana: 15,
            base_max_mana: 15,
            gold: 0,
            owned_attacks,
            equipped_attacks,
            inventory,
            equipped_trinkets: [None; MAX_TRINKETS],
        }
    }

    pub fn max_hp(&self) -> u32 {
        self.base_max_hp + self.trinket_max_hp_bonus()
    }

    pub fn max_mana(&self) -> u32 {
        self.base_max_mana + self.trinket_max_mana_bonus()
    }

    pub fn trinket_max_hp_bonus(&self) -> u32 {
        self.equipped_trinkets
            .iter()
            .filter_map(|t| t.as_ref())
            .map(|t| t.bonus_max_hp())
            .sum()
    }

    pub fn trinket_max_mana_bonus(&self) -> u32 {
        self.equipped_trinkets
            .iter()
            .filter_map(|t| t.as_ref())
            .map(|t| t.bonus_max_mana())
            .sum()
    }

    pub fn equipped_attacks_resolved(&self) -> Vec<Option<&Attack>> {
        self.equipped_attacks
            .iter()
            .map(|slot| slot.and_then(|idx| self.owned_attacks.get(idx)))
            .collect()
    }

    pub fn assign_attack_to_slot(&mut self, attack_idx: usize, slot: usize) {
        if slot >= MAX_ATTACKS || attack_idx >= self.owned_attacks.len() {
            return;
        }
        for s in self.equipped_attacks.iter_mut() {
            if *s == Some(attack_idx) {
                *s = None;
            }
        }
        self.equipped_attacks[slot] = Some(attack_idx);
    }
}
