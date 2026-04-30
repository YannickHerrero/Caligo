use super::attack::Element;
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
    pub color: Color,
    pub is_boss: bool,
    pub description: String,
}

impl Enemy {
    pub fn slime() -> Self {
        let sprite = vec![
            "   _____   ".to_string(),
            "  /     \\  ".to_string(),
            " | o   o | ".to_string(),
            "  \\__~__/  ".to_string(),
        ];
        Self {
            name: "Slime".to_string(),
            primary_type: Element::Water,
            secondary_type: None,
            hp: 30,
            max_hp: 30,
            speed: 12,
            moveset: vec!["Splash", "Bubble"],
            sprite,
            color: Color::Rgb(120, 200, 220),
            is_boss: false,
            description: "A wobbling blob of seawater. Bops more than it bites.".to_string(),
        }
    }
}
