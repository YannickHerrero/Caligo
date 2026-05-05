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
}

#[derive(Debug, Clone)]
pub enum Item {
    HpPotion(PotionSize),
    ManaPotion(PotionSize),
    AttackStone { attack_name: String },
    Trinket(TrinketKind),
    Utility(UtilityKind),
    /// Single-use capture device. Consumed at the post-fight prompt
    /// regardless of catch success.
    MonsterNet,
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
            Item::MonsterNet => "Monster Net".to_string(),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Item::HpPotion(_) => Color::Rgb(255, 120, 120),
            Item::ManaPotion(_) => Color::Rgb(120, 160, 255),
            Item::AttackStone { .. } => Color::Rgb(200, 180, 120),
            Item::Trinket(_) => Color::Rgb(220, 180, 255),
            Item::Utility(_) => Color::Rgb(200, 200, 160),
            Item::MonsterNet => Color::Rgb(160, 220, 200),
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
