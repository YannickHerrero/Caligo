use super::projectile::ProjectileKind;

pub const MAX_ATTACKS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Jump,
    Dash,
    Throw(ProjectileKind),
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub name: String,
    pub kind: AnimationKind,
}

impl Attack {
    pub fn new(name: &str, kind: AnimationKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
        }
    }
}
