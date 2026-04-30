use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    Hearts,
    Triangles,
    Circles,
}

impl ParticleKind {
    pub fn glyph(&self) -> &'static str {
        match self {
            ParticleKind::Hearts => "♥",
            ParticleKind::Triangles => "▲",
            ParticleKind::Circles => "●",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ParticleKind::Hearts => Color::Rgb(255, 150, 200),
            ParticleKind::Triangles => Color::Rgb(240, 90, 70),
            ParticleKind::Circles => Color::Rgb(110, 170, 255),
        }
    }
}
