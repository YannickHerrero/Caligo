#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Attack,
    Skill,
    Item,
    Defend,
}

impl Action {
    pub const ALL: &'static [Action] = &[
        Action::Attack,
        Action::Skill,
        Action::Item,
        Action::Defend,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Action::Attack => "Attack",
            Action::Skill => "Skill",
            Action::Item => "Item",
            Action::Defend => "Defend",
        }
    }
}
