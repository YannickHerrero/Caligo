use ratatui::style::Color;

pub struct Enemy {
    pub name: String,
    pub hp: u32,
    pub max_hp: u32,
    pub sprite: Vec<String>,
    pub color: Color,
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
            hp: 30,
            max_hp: 30,
            sprite,
            color: Color::Rgb(120, 200, 120),
        }
    }
}
