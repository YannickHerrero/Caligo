#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
}

impl Item {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
