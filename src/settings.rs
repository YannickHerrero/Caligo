use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn label(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub theme: Theme,
}

impl Settings {
    pub const DEFAULT: Settings = Settings {
        theme: Theme::Dark,
    };
}

static SETTINGS: RwLock<Settings> = RwLock::new(Settings::DEFAULT);

pub fn theme() -> Theme {
    SETTINGS.read().unwrap().theme
}

pub fn set_theme(theme: Theme) {
    SETTINGS.write().unwrap().theme = theme;
}

pub fn init_from_env() {
    if let Ok(value) = std::env::var("CALIGO_THEME") {
        let parsed = match value.to_lowercase().as_str() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        };
        if let Some(theme) = parsed {
            set_theme(theme);
        }
    }
}
