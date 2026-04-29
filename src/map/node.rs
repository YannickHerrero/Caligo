use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    EasyFight,
    NormalFight,
    EliteFight,
    Camp,
    Shop,
    Mystery,
    Boss,
}

impl NodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            NodeKind::EasyFight => "Easy Fight",
            NodeKind::NormalFight => "Normal Fight",
            NodeKind::EliteFight => "Elite Fight",
            NodeKind::Camp => "Campment",
            NodeKind::Shop => "Shop",
            NodeKind::Mystery => "Mystery",
            NodeKind::Boss => "Boss",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NodeKind::EasyFight => "A weak foe. Light reward.",
            NodeKind::NormalFight => "A standard fight.",
            NodeKind::EliteFight => "A tougher enemy. Better loot.",
            NodeKind::Camp => "Rest, heal, or sharpen your shell.",
            NodeKind::Shop => "Spend coin on goods.",
            NodeKind::Mystery => "An unknown encounter awaits.",
            NodeKind::Boss => "The floor's master. No turning back.",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NodeKind::EasyFight => "x",
            NodeKind::NormalFight => "⚔",
            NodeKind::EliteFight => "⚜",
            NodeKind::Camp => "♨",
            NodeKind::Shop => "$",
            NodeKind::Mystery => "?",
            NodeKind::Boss => "☠",
        }
    }

    pub fn card_label(&self) -> &'static str {
        match self {
            NodeKind::EasyFight => "Easy",
            NodeKind::NormalFight => "Normal",
            NodeKind::EliteFight => "Elite",
            NodeKind::Camp => "Camp",
            NodeKind::Shop => "Shop",
            NodeKind::Mystery => "Mystery",
            NodeKind::Boss => "BOSS",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            NodeKind::EasyFight => Color::Rgb(120, 200, 120),
            NodeKind::NormalFight => Color::Rgb(220, 90, 90),
            NodeKind::EliteFight => Color::Rgb(255, 150, 60),
            NodeKind::Camp => Color::Rgb(255, 210, 110),
            NodeKind::Shop => Color::Rgb(110, 210, 230),
            NodeKind::Mystery => Color::Rgb(190, 130, 230),
            NodeKind::Boss => Color::Rgb(230, 70, 130),
        }
    }
}

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub struct MapNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub floor: u8,
    pub column: u8,
    pub children: Vec<NodeId>,
    pub visited: bool,
}

impl MapNode {
    pub fn new(id: NodeId, kind: NodeKind, floor: u8, column: u8) -> Self {
        Self {
            id,
            kind,
            floor,
            column,
            children: Vec::new(),
            visited: false,
        }
    }
}
