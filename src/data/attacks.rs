use crate::fight::{AnimationKind, Attack, Element, ProjectileKind};

pub const STARTER_ATTACK_NAMES: [&str; 4] = ["Pinch", "Bubble", "Snip", "Cosmic Orb"];

pub fn all_attacks() -> Vec<Attack> {
    let mut out = Vec::new();
    out.extend(neutral());
    out.extend(fire());
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

fn fire() -> Vec<Attack> {
    vec![
        Attack::new(
            "Ember",
            AnimationKind::Throw(ProjectileKind::Fire),
            6,
            2,
            Element::Fire,
            "A small flicker of flame.",
        ),
        Attack::new(
            "Cinder Spit",
            AnimationKind::Throw(ProjectileKind::Fire),
            7,
            3,
            Element::Fire,
            "Gobbets of glowing ember.",
        ),
        Attack::new(
            "Heatwave",
            AnimationKind::Throw(ProjectileKind::Fire),
            8,
            4,
            Element::Fire,
            "A wide, shimmering burst.",
        ),
        Attack::new(
            "Flame Dash",
            AnimationKind::Dash,
            9,
            4,
            Element::Fire,
            "Wreathed in flame, charges through.",
        ),
        Attack::new(
            "Fireball",
            AnimationKind::Throw(ProjectileKind::Fire),
            11,
            5,
            Element::Fire,
            "Classic burning sphere.",
        ),
        Attack::new(
            "Sunflare",
            AnimationKind::Throw(ProjectileKind::Fire),
            13,
            6,
            Element::Fire,
            "A blinding ball of solar fire.",
        ),
        Attack::new(
            "Lava Lob",
            AnimationKind::Throw(ProjectileKind::Fire),
            14,
            6,
            Element::Fire,
            "Splashes molten rock on the foe.",
        ),
        Attack::new(
            "Pyre Charge",
            AnimationKind::Dash,
            15,
            7,
            Element::Fire,
            "A flaming charge across the field.",
        ),
        Attack::new(
            "Magma Crush",
            AnimationKind::Jump,
            17,
            8,
            Element::Fire,
            "Glowing claws crash down.",
        ),
        Attack::new(
            "Inferno",
            AnimationKind::Throw(ProjectileKind::Fire),
            21,
            10,
            Element::Fire,
            "A roaring blast of fire.",
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
