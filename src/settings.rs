use std::path::PathBuf;
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

    fn key(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
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
    let snapshot = {
        let mut s = SETTINGS.write().unwrap();
        s.theme = theme;
        *s
    };
    save_to_disk(&snapshot);
}

pub fn init() {
    if let Some(loaded) = load_from_disk() {
        *SETTINGS.write().unwrap() = loaded;
    }
    if let Ok(value) = std::env::var("CALIGO_THEME") {
        if let Some(theme) = Theme::parse(&value) {
            SETTINGS.write().unwrap().theme = theme;
        }
    }
}

fn config_path() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })?;
    Some(base.join("caligo").join("settings"))
}

fn load_from_disk() -> Option<Settings> {
    let path = config_path()?;
    let contents = std::fs::read_to_string(&path).ok()?;
    let mut settings = Settings::DEFAULT;
    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "theme" {
            if let Some(theme) = Theme::parse(value) {
                settings.theme = theme;
            }
        }
    }
    Some(settings)
}

fn save_to_disk(settings: &Settings) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let contents = format!("theme={}\n", settings.theme.key());
    let _ = std::fs::write(&path, contents);
}
