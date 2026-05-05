use crate::data::attacks as attack_lib;
use crate::data::starters::Starter;
use crate::fight::{Attack, Item, ItemStack, PotionSize, TrinketKind, UtilityKind, MAX_ATTACKS};

pub const MAX_TRINKETS: usize = 2;

pub type EquippedTrinkets = [Option<TrinketKind>; MAX_TRINKETS];

pub const PLAYER_BASE_SPEED: u32 = 10;

pub struct Player {
    pub hp: u32,
    pub base_max_hp: u32,
    pub mana: u32,
    pub base_max_mana: u32,
    pub speed: u32,
    pub gold: u32,
    pub owned_attacks: Vec<Attack>,
    pub equipped_attacks: [Option<usize>; MAX_ATTACKS],
    pub inventory: Vec<ItemStack>,
    pub equipped_trinkets: EquippedTrinkets,
}

impl Player {
    pub fn new() -> Self {
        let owned_attacks = attack_lib::all_attacks();
        let equipped_attacks = resolve_starter_slots(&owned_attacks);

        let inventory = vec![
            ItemStack::new(Item::HpPotion(PotionSize::Small), 2),
            ItemStack::new(Item::HpPotion(PotionSize::Large), 1),
            ItemStack::new(Item::ManaPotion(PotionSize::Small), 2),
            ItemStack::new(Item::ManaPotion(PotionSize::Large), 1),
            ItemStack::new(
                Item::AttackStone {
                    attack_name: "Bramble Trap".to_string(),
                },
                1,
            ),
            ItemStack::new(Item::Trinket(TrinketKind::HeartCharm), 1),
            ItemStack::new(Item::Trinket(TrinketKind::ManaPearl), 1),
            ItemStack::new(Item::Trinket(TrinketKind::LuckyShell), 1),
            ItemStack::new(Item::Utility(UtilityKind::Revive), 1),
            ItemStack::new(Item::Utility(UtilityKind::GoldPouch), 1),
        ];

        Self {
            hp: 25,
            base_max_hp: 25,
            mana: 15,
            base_max_mana: 15,
            speed: PLAYER_BASE_SPEED,
            gold: 0,
            owned_attacks,
            equipped_attacks,
            inventory,
            equipped_trinkets: [None; MAX_TRINKETS],
        }
    }

    /// Build a fresh Player for the start of a real run with the chosen
    /// starter. Stats are sourced from the starter's PartyMember view;
    /// inventory holds a single Small HP Potion and a single Monster
    /// Net.
    pub fn for_starter(starter: &Starter) -> Self {
        let id = crate::meta::starter_id(&starter.name);
        let member = crate::run::PartyMember::fresh(id, starter.clone());
        Self::for_run_party_member(&member)
    }

    /// Build a Player whose stats mirror the given (active) party
    /// member. Used at run start with the first member, and on switch.
    pub fn for_run_party_member(member: &crate::run::PartyMember) -> Self {
        let inventory = vec![
            ItemStack::new(Item::HpPotion(PotionSize::Small), 1),
            ItemStack::new(Item::MonsterNet, 1),
        ];
        let mut player = Self {
            hp: 0,
            base_max_hp: 0,
            mana: 0,
            base_max_mana: 0,
            speed: 0,
            gold: 0,
            owned_attacks: Vec::new(),
            equipped_attacks: [None; MAX_ATTACKS],
            inventory,
            equipped_trinkets: [None; MAX_TRINKETS],
        };
        player.sync_from_member(member);
        player
    }

    /// Copy a party member's stats into this Player so combat reads
    /// reflect the new active member. Called at run start and on
    /// switch-in.
    pub fn sync_from_member(&mut self, member: &crate::run::PartyMember) {
        self.hp = member.current_hp;
        self.base_max_hp = member.max_hp;
        self.mana = member.current_mana;
        self.base_max_mana = member.max_mana;
        self.speed = member.speed;
        self.owned_attacks = member.attacks.clone();
        let n = member.attacks.len().min(MAX_ATTACKS);
        let mut equipped = [None; MAX_ATTACKS];
        for i in 0..n {
            equipped[i] = Some(i);
        }
        self.equipped_attacks = equipped;
    }

    /// Write this Player's working state back into the given member.
    /// Called at fight end and on switch-out so per-member HP/MP
    /// persists.
    pub fn sync_to_member(&self, member: &mut crate::run::PartyMember) {
        member.current_hp = self.hp;
        member.current_mana = self.mana;
        // owned_attacks may have grown via an AttackStone teach; persist
        // the new list back.
        member.attacks = self.owned_attacks.clone();
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

}

fn resolve_starter_slots(attacks: &[Attack]) -> [Option<usize>; MAX_ATTACKS] {
    resolve_starter_attack_slots(attacks, &attack_lib::STARTER_ATTACK_NAMES)
}

fn resolve_starter_attack_slots(
    attacks: &[Attack],
    names: &[&str],
) -> [Option<usize>; MAX_ATTACKS] {
    let mut slots = [None; MAX_ATTACKS];
    for (slot, name) in names.iter().enumerate() {
        if slot >= MAX_ATTACKS {
            break;
        }
        slots[slot] = attacks.iter().position(|a| a.name == *name);
    }
    slots
}

