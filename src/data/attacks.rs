use crate::fight::{AnimationKind, Attack, Element, ProjectileKind};

pub const STARTER_ATTACK_NAMES: [&str; 4] = ["Pinch", "Bubble", "Snip", "Cosmic Orb"];

pub fn all_attacks() -> Vec<Attack> {
    starters()
}

fn starters() -> Vec<Attack> {
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
            "Bubble",
            AnimationKind::Throw(ProjectileKind::Water),
            7,
            3,
            Element::Water,
            "Lobs a bubble that splashes the enemy.",
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
            "Cosmic Orb",
            AnimationKind::Throw(ProjectileKind::EnergyBall),
            14,
            8,
            Element::Air,
            "A heavy orb of cosmic energy. High cost, high damage.",
        ),
    ]
}
