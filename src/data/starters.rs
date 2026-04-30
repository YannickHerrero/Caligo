use crate::crab::entity::{build_frame, BodyTemplates, Eyes, Mouths};
use crate::fight::Element;
use crate::palette::ThemedColor;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Starter {
    pub name: String,
    pub primary_type: Element,
    pub starting_attacks: Vec<&'static str>,
    pub sprite: Vec<String>,
    pub palette: ThemedColor,
    pub description: String,
}

impl Starter {
    pub fn color(&self) -> Color {
        self.palette.resolve()
    }
}

pub fn all_starters() -> Vec<Starter> {
    vec![pinchy(), cinder(), sprout()]
}

fn crab_sprite() -> Vec<String> {
    build_frame(BodyTemplates::STANDING_RIGHT, Eyes::NEUTRAL, Mouths::NEUTRAL)
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn pinchy() -> Starter {
    Starter {
        name: "Pinchy".to_string(),
        primary_type: Element::Water,
        starting_attacks: vec!["Pinch", "Bubble", "Snip", "Cosmic Orb"],
        sprite: crab_sprite(),
        palette: ThemedColor::Fixed(Color::Rgb(255, 140, 90)),
        description:
            "The default tidepool crab. Balanced and sturdy, with a generalist starting kit."
                .to_string(),
    }
}

fn cinder() -> Starter {
    Starter {
        name: "Cinder".to_string(),
        primary_type: Element::Fire,
        starting_attacks: vec!["Pinch", "Ember", "Snip", "Cinder Spit"],
        sprite: crab_sprite(),
        palette: ThemedColor::Fixed(Color::Rgb(220, 90, 50)),
        description:
            "Hatched in a tidepool that ran a little hot. Aggressive opener, fragile shell."
                .to_string(),
    }
}

fn sprout() -> Starter {
    Starter {
        name: "Sprout".to_string(),
        primary_type: Element::Grass,
        starting_attacks: vec!["Pinch", "Vine Whip", "Snip", "Leaf Slash"],
        sprite: crab_sprite(),
        palette: ThemedColor::Themed {
            dark: Color::Rgb(120, 210, 110),
            light: Color::Rgb(40, 140, 60),
        },
        description: "A crab who has clearly been spending time in the kelp. Slow but steady."
            .to_string(),
    }
}
