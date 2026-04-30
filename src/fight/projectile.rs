use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileKind {
    Water,
    Fire,
    Electric,
    EnergyBall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileSize {
    Small,
    Medium,
    Large,
}

impl ProjectileSize {
    pub fn for_damage(damage: u32) -> Self {
        match damage {
            0..=8 => ProjectileSize::Small,
            9..=16 => ProjectileSize::Medium,
            _ => ProjectileSize::Large,
        }
    }
}

impl ProjectileKind {
    pub fn sprite(&self, size: ProjectileSize) -> &'static [&'static str] {
        match (self, size) {
            (ProjectileKind::Water, ProjectileSize::Small) => &["●"],
            (ProjectileKind::Water, ProjectileSize::Medium) => &["●●", "●●"],
            (ProjectileKind::Water, ProjectileSize::Large) => &[" ● ", "●●●", " ● "],

            (ProjectileKind::Fire, ProjectileSize::Small) => &["/\\", ")("],
            (ProjectileKind::Fire, ProjectileSize::Medium) => &[" /\\", "/^\\", ")()"],
            (ProjectileKind::Fire, ProjectileSize::Large) => {
                &[" /\\ ", "/^^\\", "\\^^/", ")()("]
            }

            (ProjectileKind::Electric, ProjectileSize::Small) => &["/", "\\", "/"],
            (ProjectileKind::Electric, ProjectileSize::Medium) => &[" /", "/ ", " \\", "/ "],
            (ProjectileKind::Electric, ProjectileSize::Large) => {
                &["  /", " / ", "/  ", " \\ ", "/  "]
            }

            (ProjectileKind::EnergyBall, ProjectileSize::Small) => &[".*.", "*●*", ".*."],
            (ProjectileKind::EnergyBall, ProjectileSize::Medium) => {
                &["..*..", ".***.", "**●**", ".***.", "..*.."]
            }
            (ProjectileKind::EnergyBall, ProjectileSize::Large) => &[
                "...*...",
                "..***..",
                ".*****.",
                "**●●●**",
                ".*****.",
                "..***..",
                "...*...",
            ],
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

    pub fn width(&self, size: ProjectileSize) -> usize {
        self.sprite(size)
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0)
    }

    pub fn height(&self, size: ProjectileSize) -> usize {
        self.sprite(size).len()
    }
}
