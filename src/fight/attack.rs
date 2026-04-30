use super::particle::ParticleKind;
use super::projectile::ProjectileKind;
use ratatui::style::Color;

pub const MAX_ATTACKS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Jump,
    Dash,
    Throw(ProjectileKind),
    SelfCast(ParticleKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffKind {
    AttackUp,
    DefenseUp,
}

impl BuffKind {
    pub fn label(&self) -> &'static str {
        match self {
            BuffKind::AttackUp => "ATK",
            BuffKind::DefenseUp => "DEF",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Damage(u32),
    Heal(u32),
    Buff {
        kind: BuffKind,
        magnitude: u32,
        duration: u32,
    },
}

impl Effect {
    pub fn label(&self) -> String {
        match self {
            Effect::Damage(n) => format!("DMG {}", n),
            Effect::Heal(n) => format!("HEAL {}", n),
            Effect::Buff {
                kind,
                magnitude,
                duration,
            } => format!("{} +{}% / {}t", kind.label(), magnitude, duration),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Effect::Damage(_) => Color::Rgb(255, 140, 90),
            Effect::Heal(_) => Color::Rgb(140, 230, 160),
            Effect::Buff { .. } => Color::Rgb(230, 200, 120),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Normal,
    Fire,
    Water,
    Grass,
    Ice,
    Electric,
    Ground,
    Flying,
    Psychic,
}

impl Element {
    pub fn label(&self) -> &'static str {
        match self {
            Element::Normal => "Normal",
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Grass => "Grass",
            Element::Ice => "Ice",
            Element::Electric => "Electric",
            Element::Ground => "Ground",
            Element::Flying => "Flying",
            Element::Psychic => "Psychic",
        }
    }

    pub fn color(&self) -> Color {
        use crate::settings::{theme, Theme};
        match self {
            Element::Normal => Color::Gray,
            Element::Fire => Color::Rgb(255, 140, 60),
            Element::Water => Color::Rgb(100, 180, 255),
            Element::Grass => match theme() {
                Theme::Dark => Color::Rgb(120, 210, 110),
                Theme::Light => Color::Rgb(40, 140, 60),
            },
            Element::Ice => match theme() {
                Theme::Dark => Color::Rgb(150, 220, 240),
                Theme::Light => Color::Rgb(60, 150, 190),
            },
            Element::Electric => match theme() {
                Theme::Dark => Color::Rgb(255, 240, 100),
                Theme::Light => Color::Rgb(190, 150, 20),
            },
            Element::Ground => Color::Rgb(170, 130, 80),
            Element::Flying => match theme() {
                Theme::Dark => Color::Rgb(190, 230, 240),
                Theme::Light => Color::Rgb(70, 130, 180),
            },
            Element::Psychic => Color::Rgb(230, 110, 200),
        }
    }

    /// Damage multiplier for an attack of `self` type hitting a defender of `defender` type.
    /// Pokémon-shaped chart, with 0x immunities collapsed to 0.5x.
    pub fn effectiveness_against(self, defender: Element) -> f32 {
        use Element::*;
        match (self, defender) {
            (Fire, Ice) | (Fire, Grass) => 2.0,
            (Fire, Fire) | (Fire, Water) => 0.5,

            (Water, Fire) | (Water, Ground) => 2.0,
            (Water, Water) | (Water, Grass) => 0.5,

            (Grass, Water) | (Grass, Ground) => 2.0,
            (Grass, Fire)
            | (Grass, Grass)
            | (Grass, Ice)
            | (Grass, Flying)
            | (Grass, Electric) => 0.5,

            (Ice, Ground) | (Ice, Flying) | (Ice, Grass) => 2.0,
            (Ice, Fire) | (Ice, Water) | (Ice, Ice) => 0.5,

            (Electric, Water) | (Electric, Flying) => 2.0,
            (Electric, Electric) | (Electric, Ground) | (Electric, Grass) => 0.5,

            (Ground, Fire) | (Ground, Electric) => 2.0,
            (Ground, Flying) | (Ground, Grass) => 0.5,

            (Flying, Grass) => 2.0,
            (Flying, Electric) => 0.5,

            (Psychic, Psychic) => 0.5,

            _ => 1.0,
        }
    }

    /// Effective multiplier when the defender carries a primary and optional secondary type.
    /// Multiplies the per-type multipliers (e.g. 2x against both = 4x; 2x and 0.5x = 1x).
    pub fn effectiveness_vs(
        self,
        primary: Element,
        secondary: Option<Element>,
    ) -> f32 {
        let primary_mult = self.effectiveness_against(primary);
        let secondary_mult = secondary
            .map(|s| self.effectiveness_against(s))
            .unwrap_or(1.0);
        primary_mult * secondary_mult
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub name: String,
    pub kind: AnimationKind,
    pub effect: Effect,
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
        Self::with_effect(
            name,
            kind,
            Effect::Damage(damage),
            mana_cost,
            element,
            description,
        )
    }

    pub fn with_effect(
        name: &str,
        kind: AnimationKind,
        effect: Effect,
        mana_cost: u32,
        element: Element,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            kind,
            effect,
            mana_cost,
            element,
            description: description.to_string(),
        }
    }
}
