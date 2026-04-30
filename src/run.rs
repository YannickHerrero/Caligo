use crate::data::starters::Starter;
use crate::map::MapGraph;

/// One end-to-end playthrough of the dungeon.
///
/// Holds the chosen starter and the generated map graph. A single `Run` is
/// created by the StarterSelectScreen and lives across all map / fight /
/// reward screens until it ends (boss defeated, player dies, or the player
/// abandons). More progression fields (fights cleared, gold) get added as
/// those systems land.
#[derive(Clone)]
pub struct Run {
    pub starter: Starter,
    pub map: MapGraph,
}

impl Run {
    pub fn new(starter: Starter, map: MapGraph) -> Self {
        Self { starter, map }
    }
}
