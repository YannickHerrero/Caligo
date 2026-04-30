use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    Hearts,
    Triangles,
    Circles,
    FireSpark,
    WaterDroplet,
    EarthDust,
    AirWisp,
}

impl ParticleKind {
    pub fn glyph(&self) -> &'static str {
        match self {
            ParticleKind::Hearts => "♥",
            ParticleKind::Triangles => "▲",
            ParticleKind::Circles => "●",
            ParticleKind::FireSpark => "*",
            ParticleKind::WaterDroplet => ".",
            ParticleKind::EarthDust => ",",
            ParticleKind::AirWisp => "~",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ParticleKind::Hearts => Color::Rgb(255, 150, 200),
            ParticleKind::Triangles => Color::Rgb(240, 90, 70),
            ParticleKind::Circles => Color::Rgb(110, 170, 255),
            ParticleKind::FireSpark => Color::Rgb(255, 140, 60),
            ParticleKind::WaterDroplet => Color::Rgb(100, 180, 255),
            ParticleKind::EarthDust => Color::Rgb(170, 130, 80),
            ParticleKind::AirWisp => Color::Rgb(190, 230, 240),
        }
    }
}
