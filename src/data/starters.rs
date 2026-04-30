use crate::fight::Element;
use crate::palette::ThemedColor;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub enum StarterVisual {
    /// Render via the live Crab entity so it animates and bobs.
    AnimatedCrab,
    /// Render this fixed ASCII art (each line padded to a consistent width).
    Static(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Starter {
    pub name: String,
    pub primary_type: Element,
    pub starting_attacks: Vec<&'static str>,
    pub visual: StarterVisual,
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

fn pinchy() -> Starter {
    Starter {
        name: "Pinchy".to_string(),
        primary_type: Element::Water,
        starting_attacks: vec!["Pinch", "Bubble", "Snip", "Cosmic Orb"],
        visual: StarterVisual::AnimatedCrab,
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
        visual: StarterVisual::Static(vec![
            "    .^.    ".to_string(),
            "   /^^^\\   ".to_string(),
            "  / o o \\  ".to_string(),
            "  \\  v  /  ".to_string(),
            "   '___'   ".to_string(),
        ]),
        palette: ThemedColor::Fixed(Color::Rgb(220, 90, 50)),
        description:
            "A spry flame with a face. Aggressive opener, fragile shell.".to_string(),
    }
}

fn sprout() -> Starter {
    Starter {
        name: "Sprout".to_string(),
        primary_type: Element::Grass,
        starting_attacks: vec!["Pinch", "Vine Whip", "Snip", "Leaf Slash"],
        visual: StarterVisual::Static(vec![
            "   .---.   ".to_string(),
            "  /     \\  ".to_string(),
            "  |\\v^v/|  ".to_string(),
            "  \\_____/  ".to_string(),
            "    | |    ".to_string(),
            "   ~| |~   ".to_string(),
        ]),
        palette: ThemedColor::Themed {
            dark: Color::Rgb(120, 210, 110),
            light: Color::Rgb(40, 140, 60),
        },
        description:
            "A piranha plant on a stout stem. Patient — bites when you're not looking."
                .to_string(),
    }
}
