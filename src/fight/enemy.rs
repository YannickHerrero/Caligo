use super::attack::Element;
use crate::palette::ThemedColor;
use ratatui::style::Color;

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
    pub palette: ThemedColor,
    pub is_boss: bool,
    pub description: String,
}

impl Enemy {
    pub fn color(&self) -> Color {
        self.palette.resolve()
    }
}
