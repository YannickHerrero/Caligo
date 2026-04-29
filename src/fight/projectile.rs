use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileKind {
    Water,
    Fire,
    Electric,
    EnergyBall,
}

impl ProjectileKind {
    pub fn sprite(&self) -> &'static [&'static str] {
        match self {
            ProjectileKind::Water => &["●"],
            ProjectileKind::Fire => &["/\\", ")("],
            ProjectileKind::Electric => &["/", "\\", "/"],
            ProjectileKind::EnergyBall => &[".*.", "*●*", ".*."],
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ProjectileKind::Water => Color::Rgb(100, 180, 255),
            ProjectileKind::Fire => Color::Rgb(255, 140, 60),
            ProjectileKind::Electric => Color::Rgb(255, 240, 100),
            ProjectileKind::EnergyBall => Color::Rgb(200, 120, 255),
        }
    }

    pub fn width(&self) -> usize {
        self.sprite()
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.sprite().len()
    }
}
