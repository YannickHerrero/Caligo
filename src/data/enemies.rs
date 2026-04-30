use crate::fight::{Element, Enemy, EnemyColor};
use ratatui::style::Color;

pub fn all_enemies() -> Vec<Enemy> {
    vec![
        slime(),
        fire_slime(),
        frost_slime(),
        sandling(),
        crab_king(),
        wisp(),
        volt_wisp(),
        mind_wisp(),
        wisp_lord(),
    ]
}

pub fn slime() -> Enemy {
    Enemy {
        name: "Slime".to_string(),
        primary_type: Element::Water,
        secondary_type: None,
        hp: 30,
        max_hp: 30,
        speed: 12,
        moveset: vec!["Splash", "Bubble"],
        sprite: vec![
            "   _____   ".to_string(),
            "  /     \\  ".to_string(),
            " | o   o | ".to_string(),
            "  \\__~__/  ".to_string(),
        ],
        palette: EnemyColor::Fixed(Color::Rgb(120, 200, 220)),
        is_boss: false,
        description: "A wobbling blob of seawater. Bops more than it bites.".to_string(),
    }
}

pub fn fire_slime() -> Enemy {
    Enemy {
        name: "Fire Slime".to_string(),
        primary_type: Element::Fire,
        secondary_type: None,
        hp: 28,
        max_hp: 28,
        speed: 14,
        moveset: vec!["Ember", "Cinder Spit"],
        sprite: vec![
            "   \\v_v/   ".to_string(),
            "  / *o* \\  ".to_string(),
            " | >   < | ".to_string(),
            "  \\_~~~_/  ".to_string(),
        ],
        palette: EnemyColor::Fixed(Color::Rgb(220, 100, 60)),
        is_boss: false,
        description: "Hot to the touch. Leaves scorch marks where it scoots.".to_string(),
    }
}

pub fn frost_slime() -> Enemy {
    Enemy {
        name: "Frost Slime".to_string(),
        primary_type: Element::Ice,
        secondary_type: None,
        hp: 32,
        max_hp: 32,
        speed: 8,
        moveset: vec!["Frostbite", "Ice Shard"],
        sprite: vec![
            "   *_/\\_*  ".to_string(),
            "  /     \\  ".to_string(),
            " | x   x | ".to_string(),
            "  \\..-../  ".to_string(),
        ],
        palette: EnemyColor::Themed {
            dark: Color::Rgb(180, 220, 255),
            light: Color::Rgb(60, 140, 190),
        },
        is_boss: false,
        description: "Half ice, all attitude. Slows down the unwary.".to_string(),
    }
}

pub fn sandling() -> Enemy {
    Enemy {
        name: "Sandling".to_string(),
        primary_type: Element::Ground,
        secondary_type: None,
        hp: 40,
        max_hp: 40,
        speed: 6,
        moveset: vec!["Granite Shell", "Sandstorm", "Stone Slam"],
        sprite: vec![
            "   .---.   ".to_string(),
            "  /=====\\  ".to_string(),
            " | o   o | ".to_string(),
            "  \\\\___//  ".to_string(),
        ],
        palette: EnemyColor::Fixed(Color::Rgb(200, 170, 110)),
        is_boss: false,
        description: "A pebble that decided to walk. Surprisingly tough.".to_string(),
    }
}

pub fn crab_king() -> Enemy {
    Enemy {
        name: "Crab King".to_string(),
        primary_type: Element::Water,
        secondary_type: Some(Element::Ground),
        hp: 120,
        max_hp: 120,
        speed: 10,
        moveset: vec!["Tidal Slam", "Stone Slam", "Boulder Press", "Tsunami"],
        sprite: vec![
            "   _,---,_   ".to_string(),
            "  / *   * \\  ".to_string(),
            "((  >   <  ))".to_string(),
            "  \\\\__o__//  ".to_string(),
            "  '-------'  ".to_string(),
        ],
        palette: EnemyColor::Fixed(Color::Rgb(180, 60, 60)),
        is_boss: true,
        description: "An ancient ruler of the tidepools, returned to claim what is his.".to_string(),
    }
}

pub fn wisp() -> Enemy {
    Enemy {
        name: "Wisp".to_string(),
        primary_type: Element::Flying,
        secondary_type: None,
        hp: 22,
        max_hp: 22,
        speed: 18,
        moveset: vec!["Gust", "Tornado"],
        sprite: vec![
            "   .---.   ".to_string(),
            "  / o o \\  ".to_string(),
            "  \\  -  /  ".to_string(),
            "   '. .'   ".to_string(),
        ],
        palette: EnemyColor::Themed {
            dark: Color::Rgb(220, 230, 245),
            light: Color::Rgb(90, 120, 160),
        },
        is_boss: false,
        description: "Drifts through caves on wind that isn't quite there.".to_string(),
    }
}

pub fn volt_wisp() -> Enemy {
    Enemy {
        name: "Volt Wisp".to_string(),
        primary_type: Element::Electric,
        secondary_type: None,
        hp: 24,
        max_hp: 24,
        speed: 20,
        moveset: vec!["Spark", "Thunderclap"],
        sprite: vec![
            "   .-^-.   ".to_string(),
            "  /' o '\\  ".to_string(),
            "  \\//=\\\\/  ".to_string(),
            "   '! !'   ".to_string(),
        ],
        palette: EnemyColor::Themed {
            dark: Color::Rgb(255, 230, 80),
            light: Color::Rgb(180, 130, 10),
        },
        is_boss: false,
        description: "Crackles with static. Sparks fly when it bumps into things.".to_string(),
    }
}

pub fn mind_wisp() -> Enemy {
    Enemy {
        name: "Mind Wisp".to_string(),
        primary_type: Element::Psychic,
        secondary_type: None,
        hp: 26,
        max_hp: 26,
        speed: 16,
        moveset: vec!["Gust", "Cosmic Orb"],
        sprite: vec![
            "   .* *.   ".to_string(),
            "  ( o o )  ".to_string(),
            "   ' v '   ".to_string(),
            "   ~ . ~   ".to_string(),
        ],
        palette: EnemyColor::Fixed(Color::Rgb(220, 130, 220)),
        is_boss: false,
        description: "Watches you from a distance you can't quite measure.".to_string(),
    }
}

pub fn wisp_lord() -> Enemy {
    Enemy {
        name: "Wisp Lord".to_string(),
        primary_type: Element::Flying,
        secondary_type: Some(Element::Psychic),
        hp: 100,
        max_hp: 100,
        speed: 18,
        moveset: vec!["Tornado", "Cosmic Orb", "Star Lance", "Sky Splitter"],
        sprite: vec![
            "    /\\/\\/\\/\\/    ".to_string(),
            "    <* * * *>    ".to_string(),
            "    /       \\    ".to_string(),
            "   | O     O |   ".to_string(),
            "   |   \\v/   |   ".to_string(),
            "    \\  ___  /    ".to_string(),
            "     '-----'     ".to_string(),
        ],
        palette: EnemyColor::Fixed(Color::Rgb(150, 100, 200)),
        is_boss: true,
        description: "The eldest of the wisps. Its gaze pries memories loose.".to_string(),
    }
}
