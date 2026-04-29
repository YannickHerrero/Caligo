use super::projectile::ProjectileKind;
use ratatui::style::Color;

pub const MAX_ATTACKS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Jump,
    Dash,
    Throw(ProjectileKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Neutral,
    Fire,
    Water,
    Earth,
    Air,
}

impl Element {
    pub fn label(&self) -> &'static str {
        match self {
            Element::Neutral => "Neutral",
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Air => "Air",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Element::Neutral => Color::Gray,
            Element::Fire => Color::Rgb(255, 140, 60),
            Element::Water => Color::Rgb(100, 180, 255),
            Element::Earth => Color::Rgb(170, 130, 80),
            Element::Air => Color::Rgb(190, 230, 240),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub name: String,
    pub kind: AnimationKind,
    pub damage: u32,
    pub mana_cost: u32,
    pub element: Element,
    pub description: String,
}

impl Attack {
    pub fn new(
        name: &str,
        kind: AnimationKind,
        damage: u32,
        mana_cost: u32,
        element: Element,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            kind,
            damage,
            mana_cost,
            element,
            description: description.to_string(),
        }
    }
}
