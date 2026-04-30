use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionSize {
    Small,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrinketKind {
    HeartCharm,
    ManaPearl,
    LuckyShell,
}

impl TrinketKind {
    pub fn name(&self) -> &'static str {
        match self {
            TrinketKind::HeartCharm => "Heart Charm",
            TrinketKind::ManaPearl => "Mana Pearl",
            TrinketKind::LuckyShell => "Lucky Shell",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TrinketKind::HeartCharm => "+10 max HP while equipped.",
            TrinketKind::ManaPearl => "+5 max MP while equipped.",
            TrinketKind::LuckyShell => "Slight luck bonus while equipped.",
        }
    }

    pub fn bonus_max_hp(&self) -> u32 {
        match self {
            TrinketKind::HeartCharm => 10,
            _ => 0,
        }
    }

    pub fn bonus_max_mana(&self) -> u32 {
        match self {
            TrinketKind::ManaPearl => 5,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityKind {
    Revive,
    GoldPouch,
}

impl UtilityKind {
    pub fn name(&self) -> &'static str {
        match self {
            UtilityKind::Revive => "Revive Pearl",
            UtilityKind::GoldPouch => "Gold Pouch",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            UtilityKind::Revive => "Auto-revives on defeat (used in combat).",
            UtilityKind::GoldPouch => "Contains 25 gold. Open to claim.",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    HpPotion(PotionSize),
    ManaPotion(PotionSize),
    AttackStone { attack_name: String },
    Trinket(TrinketKind),
    Utility(UtilityKind),
}

impl Item {
    pub fn name(&self) -> String {
        match self {
            Item::HpPotion(PotionSize::Small) => "Small HP Potion".to_string(),
            Item::HpPotion(PotionSize::Large) => "Large HP Potion".to_string(),
            Item::ManaPotion(PotionSize::Small) => "Small Mana Potion".to_string(),
            Item::ManaPotion(PotionSize::Large) => "Large Mana Potion".to_string(),
            Item::AttackStone { attack_name } => format!("Stone of {}", attack_name),
            Item::Trinket(t) => t.name().to_string(),
            Item::Utility(u) => u.name().to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Item::HpPotion(PotionSize::Small) => "Restores 10 HP.".to_string(),
            Item::HpPotion(PotionSize::Large) => "Restores 30 HP.".to_string(),
            Item::ManaPotion(PotionSize::Small) => "Restores 6 MP.".to_string(),
            Item::ManaPotion(PotionSize::Large) => "Restores 15 MP.".to_string(),
            Item::AttackStone { attack_name } => {
                format!("Teaches the attack '{}' when used.", attack_name)
            }
            Item::Trinket(t) => t.description().to_string(),
            Item::Utility(u) => u.description().to_string(),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Item::HpPotion(_) => Color::Rgb(255, 120, 120),
            Item::ManaPotion(_) => Color::Rgb(120, 160, 255),
            Item::AttackStone { .. } => Color::Rgb(200, 180, 120),
            Item::Trinket(_) => Color::Rgb(220, 180, 255),
            Item::Utility(_) => Color::Rgb(200, 200, 160),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item: Item, count: u32) -> Self {
        Self { item, count }
    }
}
