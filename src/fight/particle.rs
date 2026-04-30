use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    Hearts,
    Triangles,
    Circles,
    FireSpark,
    WaterDroplet,
    GrassLeaf,
    IceShard,
    ElectricSpark,
    GroundDust,
    FlyingWisp,
    PsychicSpark,
    NormalHit,
}

impl ParticleKind {
    pub fn glyph(&self) -> &'static str {
        match self {
            ParticleKind::Hearts => "♥",
            ParticleKind::Triangles => "▲",
            ParticleKind::Circles => "●",
            ParticleKind::FireSpark => "*",
            ParticleKind::WaterDroplet => ".",
            ParticleKind::GrassLeaf => "v",
            ParticleKind::IceShard => "+",
            ParticleKind::ElectricSpark => "'",
            ParticleKind::GroundDust => ",",
            ParticleKind::FlyingWisp => "~",
            ParticleKind::PsychicSpark => "°",
            ParticleKind::NormalHit => "*",
        }
    }

    pub fn color(&self) -> Color {
        use crate::settings::{theme, Theme};
        match self {
            ParticleKind::Hearts => Color::Rgb(255, 150, 200),
            ParticleKind::Triangles => Color::Rgb(240, 90, 70),
            ParticleKind::Circles => Color::Rgb(110, 170, 255),
            ParticleKind::FireSpark => Color::Rgb(255, 140, 60),
            ParticleKind::WaterDroplet => Color::Rgb(100, 180, 255),
            ParticleKind::GrassLeaf => match theme() {
                Theme::Dark => Color::Rgb(120, 210, 110),
                Theme::Light => Color::Rgb(40, 140, 60),
            },
            ParticleKind::IceShard => match theme() {
                Theme::Dark => Color::Rgb(150, 220, 240),
                Theme::Light => Color::Rgb(60, 150, 190),
            },
            ParticleKind::ElectricSpark => match theme() {
                Theme::Dark => Color::Rgb(255, 240, 100),
                Theme::Light => Color::Rgb(190, 150, 20),
            },
            ParticleKind::GroundDust => Color::Rgb(170, 130, 80),
            ParticleKind::FlyingWisp => match theme() {
                Theme::Dark => Color::Rgb(190, 230, 240),
                Theme::Light => Color::Rgb(70, 130, 180),
            },
            ParticleKind::PsychicSpark => Color::Rgb(230, 110, 200),
            ParticleKind::NormalHit => match theme() {
                Theme::Dark => Color::Rgb(220, 220, 220),
                Theme::Light => Color::Rgb(100, 100, 100),
            },
        }
    }
}
