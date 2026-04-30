use super::attack::Element;
use crate::settings::{theme, Theme};
use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub enum EnemyColor {
    Fixed(Color),
    Themed { dark: Color, light: Color },
}

impl EnemyColor {
    pub fn resolve(&self) -> Color {
        match self {
            EnemyColor::Fixed(c) => *c,
            EnemyColor::Themed { dark, light } => match theme() {
                Theme::Dark => *dark,
                Theme::Light => *light,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub name: String,
    pub primary_type: Element,
    pub secondary_type: Option<Element>,
    pub hp: u32,
    pub max_hp: u32,
    pub speed: u32,
    pub moveset: Vec<&'static str>,
    pub sprite: Vec<String>,
    pub palette: EnemyColor,
    pub is_boss: bool,
    pub description: String,
}

impl Enemy {
    pub fn color(&self) -> Color {
        self.palette.resolve()
    }
}
