#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Ecstatic,
    Happy,
    Neutral,
    Sad,
    Hungry,
}

impl Mood {
    pub fn from_happiness(happiness: u8) -> Self {
        match happiness {
            90..=100 => Mood::Ecstatic,
            70..=89 => Mood::Happy,
            40..=69 => Mood::Neutral,
            20..=39 => Mood::Sad,
            _ => Mood::Hungry,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Mood::Ecstatic => "Ecstatic",
            Mood::Happy => "Happy",
            Mood::Neutral => "Neutral",
            Mood::Sad => "Sad",
            Mood::Hungry => "Hungry",
        }
    }

    pub fn animation_speed(&self) -> f32 {
        match self {
            Mood::Ecstatic => 2.0,
            Mood::Happy => 1.0,
            Mood::Neutral => 0.6,
            Mood::Sad => 0.3,
            Mood::Hungry => 0.2,
        }
    }
}

impl std::fmt::Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
