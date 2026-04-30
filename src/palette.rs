use crate::settings::{theme, Theme};
use ratatui::style::Color;

/// A color that can either stay fixed across themes or pick a different shade per theme.
#[derive(Debug, Clone, Copy)]
pub enum ThemedColor {
    Fixed(Color),
    Themed { dark: Color, light: Color },
}

impl ThemedColor {
    pub fn resolve(&self) -> Color {
        match self {
            ThemedColor::Fixed(c) => *c,
            ThemedColor::Themed { dark, light } => match theme() {
                Theme::Dark => *dark,
                Theme::Light => *light,
            },
        }
    }
}
