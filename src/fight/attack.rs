pub const MAX_ATTACKS: usize = 4;

#[derive(Debug, Clone)]
pub struct Attack {
    pub name: String,
}

impl Attack {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
