use crate::fight::{Element, Enemy};
use ratatui::style::Color;

pub fn all_enemies() -> Vec<Enemy> {
    vec![slime(), fire_slime(), frost_slime(), sandling(), crab_king()]
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
        color: Color::Rgb(120, 200, 220),
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
        color: Color::Rgb(220, 100, 60),
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
        color: Color::Rgb(180, 220, 255),
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
        color: Color::Rgb(200, 170, 110),
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
        color: Color::Rgb(180, 60, 60),
        is_boss: true,
        description: "An ancient ruler of the tidepools, returned to claim what is his.".to_string(),
    }
}
