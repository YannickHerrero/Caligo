use crate::data::starters::Starter;
use crate::map::MapGraph;
use crate::meta::MonsterId;

/// A single member of the player's run-time party. Today only starters
/// can occupy a slot; wild captures will land in Phase 4 with an
/// expanded template type.
#[derive(Clone)]
pub struct PartyMember {
    pub id: MonsterId,
    /// Species template (visual / type / starting attacks). Will widen
    /// to a `MonsterTemplate` once wild monsters can be party members.
    pub template: Starter,
}

impl PartyMember {
    pub fn from_starter(id: MonsterId, starter: Starter) -> Self {
        Self { id, template: starter }
    }
}

/// One end-to-end playthrough of the dungeon.
///
/// Holds the player's party for the run and the generated map graph. A
/// single `Run` is created when leaving the start menu and lives across
/// all map / fight / reward screens until it ends (boss defeated, player
/// dies, or the player abandons).
#[derive(Clone)]
pub struct Run {
    pub party: Vec<PartyMember>,
    /// Index of the currently-active party member. The fight screen
    /// reads stats from `party[active]`; switching/faint events change
    /// this.
    pub active: usize,
    pub map: MapGraph,
}

impl Run {
    pub fn new(party: Vec<PartyMember>, map: MapGraph) -> Self {
        Self {
            party,
            active: 0,
            map,
        }
    }

    /// Convenience accessor for the active member.
    pub fn active_member(&self) -> &PartyMember {
        &self.party[self.active]
    }
}
