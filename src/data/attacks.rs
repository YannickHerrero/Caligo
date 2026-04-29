use crate::fight::{AnimationKind, Attack, Element, ProjectileKind};

pub const STARTER_ATTACK_NAMES: [&str; 4] = ["Pinch", "Bubble", "Snip", "Cosmic Orb"];

pub fn all_attacks() -> Vec<Attack> {
    let mut out = Vec::new();
    out.extend(neutral());
    out.extend(water());
    out.extend(air());
    out
}

fn neutral() -> Vec<Attack> {
    vec![
        Attack::new(
            "Pinch",
            AnimationKind::Dash,
            5,
            0,
            Element::Neutral,
            "A quick claw pinch. No mana cost, modest damage.",
        ),
        Attack::new(
            "Snip",
            AnimationKind::Jump,
            8,
            2,
            Element::Neutral,
            "Leaping snip with both claws.",
        ),
        Attack::new(
            "Scuttle Strike",
            AnimationKind::Dash,
            4,
            0,
            Element::Neutral,
            "Quick sideways jab. Free, but light.",
        ),
        Attack::new(
            "Headbutt",
            AnimationKind::Dash,
            6,
            0,
            Element::Neutral,
            "A blunt charge with the shell.",
        ),
        Attack::new(
            "Tail Whip",
            AnimationKind::Dash,
            5,
            1,
            Element::Neutral,
            "Sweeping rear-leg trip.",
        ),
        Attack::new(
            "Bite",
            AnimationKind::Dash,
            7,
            1,
            Element::Neutral,
            "A nasty mandible chomp.",
        ),
        Attack::new(
            "Shell Bash",
            AnimationKind::Dash,
            9,
            3,
            Element::Neutral,
            "Spins shell-first into the foe.",
        ),
        Attack::new(
            "Claw Crush",
            AnimationKind::Jump,
            12,
            4,
            Element::Neutral,
            "Heavy two-clawed slam.",
        ),
        Attack::new(
            "Double Snip",
            AnimationKind::Jump,
            14,
            6,
            Element::Neutral,
            "Two leaping snips in quick succession.",
        ),
        Attack::new(
            "Final Pinch",
            AnimationKind::Jump,
            18,
            7,
            Element::Neutral,
            "Double-claw guillotine. Rare and brutal.",
        ),
    ]
}

fn water() -> Vec<Attack> {
    vec![Attack::new(
        "Bubble",
        AnimationKind::Throw(ProjectileKind::Water),
        7,
        3,
        Element::Water,
        "Lobs a bubble that splashes the enemy.",
    )]
}

fn air() -> Vec<Attack> {
    vec![Attack::new(
        "Cosmic Orb",
        AnimationKind::Throw(ProjectileKind::EnergyBall),
        14,
        8,
        Element::Air,
        "A heavy orb of cosmic energy. High cost, high damage.",
    )]
}
