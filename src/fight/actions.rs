#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Attack,
    Item,
    Switch,
    Flee,
}

impl Action {
    pub const ALL: &'static [Action] =
        &[Action::Attack, Action::Item, Action::Switch, Action::Flee];

    pub fn label(&self) -> &'static str {
        match self {
            Action::Attack => "Attack",
            Action::Item => "Item",
            Action::Switch => "Switch",
            Action::Flee => "Flee",
        }
    }
}
