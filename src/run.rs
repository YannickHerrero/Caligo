use crate::map::MapGraph;

/// One end-to-end playthrough of the dungeon.
///
/// Holds the generated map graph. A single `Run` is created by the
/// StarterSelectScreen and lives across all map / fight / reward screens
/// until it ends (boss defeated, player dies, or the player abandons).
/// More fields (starter, fights cleared, gold) get added as those systems
/// land.
pub struct Run {
    pub map: MapGraph,
}

impl Run {
    pub fn new(map: MapGraph) -> Self {
        Self { map }
    }
}
