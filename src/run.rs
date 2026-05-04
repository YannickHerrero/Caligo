use crate::data::attacks as attack_lib;
use crate::data::starters::Starter;
use crate::fight::Attack;
use crate::map::MapGraph;
use crate::meta::{self, MonsterId};

const BASE_MAX_HP: u32 = 25;
const BASE_MAX_MANA: u32 = 15;
const BASE_SPEED: u32 = 10;

/// A single member of the player's run-time party. Owns its own
/// persistent HP/MP across fights within a run; resets to full at run
/// start.
#[derive(Clone)]
pub struct PartyMember {
    /// Stable identity referencing Meta.monsters / Meta.monster_ranks.
    /// Currently only used at construction time (PartyMember::fresh
    /// reads ranks via this id), but kept on the struct for re-syncs
    /// when ladders are bought mid-run, party serialisation, etc.
    #[allow(dead_code)]
    pub id: MonsterId,
    /// Species template (visual / type / starting attacks). Today this
    /// is always a `Starter`; captured wilds get a synthesised wrapper
    /// in `ui::screens::capture` until a unified MonsterTemplate lands.
    pub template: Starter,
    pub current_hp: u32,
    pub current_mana: u32,
    pub max_hp: u32,
    pub max_mana: u32,
    pub speed: u32,
    /// Permanent meta-shop boost from this member's Sharpened Edge rank.
    /// Stored as a fraction (0.40 = +40% damage).
    pub attack_boost_pct: f32,
    /// Per-member attack list. Built from `template.starting_attacks`
    /// at construction time; stones taught mid-run will append here.
    pub attacks: Vec<Attack>,
}

impl PartyMember {
    /// Build a fresh PartyMember at full HP/MP, applying the meta-shop
    /// rank investment for its MonsterId.
    pub fn fresh(id: MonsterId, template: Starter) -> Self {
        let ranks = meta::ranks_for(&id);
        let max_hp = BASE_MAX_HP + ranks.tidepool * 2;
        let max_mana = BASE_MAX_MANA + ranks.wellspring;
        let speed = BASE_SPEED + ranks.quickfoot;
        let attack_boost_pct = ranks.sharpened_edge as f32 * 0.20;
        let attacks: Vec<Attack> = template
            .starting_attacks
            .iter()
            .filter_map(|name| attack_lib::find_by_name(name))
            .collect();
        Self {
            id,
            template,
            current_hp: max_hp,
            current_mana: max_mana,
            max_hp,
            max_mana,
            speed,
            attack_boost_pct,
            attacks,
        }
    }

    /// Backwards-compat constructor for the few places that build a
    /// member without immediately needing per-member stats. Equivalent
    /// to `fresh`.
    pub fn from_starter(id: MonsterId, starter: Starter) -> Self {
        Self::fresh(id, starter)
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
