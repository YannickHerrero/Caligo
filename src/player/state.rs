use crate::fight::{
    AnimationKind, Attack, Element, Item, ItemStack, PotionSize, ProjectileKind, TrinketKind,
    UtilityKind, MAX_ATTACKS,
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

    pub fn use_inventory_item(&mut self, idx: usize) -> ItemUseResult {
        if idx >= self.inventory.len() {
            return ItemUseResult::Nothing;
        }
        let kind_kind = self.inventory[idx].item.clone();
        match &kind_kind {
            Item::Trinket(kind) => {
                let result = self.toggle_trinket(*kind);
                return result;
            }
            Item::Utility(UtilityKind::Revive | UtilityKind::EscapeToken) => {
                return ItemUseResult::CombatOnly;
            }
            _ => {}
        }
        let consumed = self.consume_one(idx);
        match consumed {
            Some(Item::HpPotion(size)) => {
                let amount = match size {
                    PotionSize::Small => 10,
                    PotionSize::Large => 30,
                };
                self.hp = (self.hp + amount).min(self.max_hp());
                ItemUseResult::Healed { hp: amount, mana: 0 }
            }
            Some(Item::ManaPotion(size)) => {
                let amount = match size {
                    PotionSize::Small => 6,
                    PotionSize::Large => 15,
                };
                self.mana = (self.mana + amount).min(self.max_mana());
                ItemUseResult::Healed { hp: 0, mana: amount }
            }
            Some(Item::AttackStone { attack_name }) => {
                if !self.owned_attacks.iter().any(|a| a.name == attack_name) {
                    self.owned_attacks.push(Attack::new(
                        &attack_name,
                        AnimationKind::Dash,
                        6,
                        2,
                        Element::Neutral,
                        "A new attack learned from a stone.",
                    ));
                    ItemUseResult::LearnedAttack(attack_name)
                } else {
                    ItemUseResult::AlreadyKnown(attack_name)
                }
            }
            Some(Item::Utility(UtilityKind::GoldPouch)) => {
                self.gold += 25;
                ItemUseResult::GoldGained(25)
            }
            _ => ItemUseResult::Nothing,
        }
    }

    fn consume_one(&mut self, idx: usize) -> Option<Item> {
        if idx >= self.inventory.len() {
            return None;
        }
        let item = self.inventory[idx].item.clone();
        self.inventory[idx].count = self.inventory[idx].count.saturating_sub(1);
        if self.inventory[idx].count == 0 {
            self.inventory.remove(idx);
        }
        Some(item)
    }

    pub fn toggle_trinket(&mut self, kind: TrinketKind) -> ItemUseResult {
        if let Some(slot) = self
            .equipped_trinkets
            .iter()
            .position(|s| *s == Some(kind))
        {
            self.equipped_trinkets[slot] = None;
            self.hp = self.hp.min(self.max_hp());
            self.mana = self.mana.min(self.max_mana());
            return ItemUseResult::TrinketUnequipped(kind);
        }
        if let Some(slot) = self.equipped_trinkets.iter().position(|s| s.is_none()) {
            self.equipped_trinkets[slot] = Some(kind);
            return ItemUseResult::TrinketEquipped(kind);
        }
        self.equipped_trinkets[0] = Some(kind);
        ItemUseResult::TrinketEquipped(kind)
    }
}

#[derive(Debug, Clone)]
pub enum ItemUseResult {
    Nothing,
    Healed { hp: u32, mana: u32 },
    LearnedAttack(String),
    AlreadyKnown(String),
    GoldGained(u32),
    TrinketEquipped(TrinketKind),
    TrinketUnequipped(TrinketKind),
    CombatOnly,
}
