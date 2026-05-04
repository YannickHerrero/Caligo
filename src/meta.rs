//! Cross-run meta state: embers (the meta-currency), the player's
//! permanent collection of monsters and their party assignments, and
//! per-monster permanent stat ranks. Persists to a file alongside the
//! settings.
//!
//! Each owned monster has a stable `MonsterId` string key
//! (`starter:pinchy`, `wild:42`) that identifies it across the meta
//! data. Ranks are scoped per `MonsterId`; the same data shape supports
//! starters, captured wild monsters, and any future party member.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Stable identifier for an owned monster. Strings rather than a newtype
/// for trivial serialisation; format is `starter:<species>` for starter
/// monsters and `wild:<n>` for captures (n from `next_wild_id`).
pub type MonsterId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterInstance {
    pub id: MonsterId,
    /// Species name; resolves through the data registry to a template
    /// (starter visuals + attacks for starter species, enemy sprite +
    /// moveset for captured wilds).
    pub species: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Meta {
    /// Total unspent meta-currency.
    pub embers: u32,
    /// All monsters the player has permanently acquired.
    pub monsters: HashMap<MonsterId, MonsterInstance>,
    /// Active party (max 6), ordered. Each entry is a key into `monsters`.
    pub party: Vec<MonsterId>,
    /// Captures from completed or attempted runs that haven't been
    /// purchased into `monsters` yet. Accumulates across runs; emptied
    /// as the player buys them in the post-run shop.
    pub pending_captures: Vec<MonsterInstance>,
    /// Counter for minting unique `wild:<n>` IDs when a capture
    /// succeeds.
    pub next_wild_id: u32,
    /// Permanent ranks per owned monster.
    pub monster_ranks: HashMap<MonsterId, StarterRanks>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StarterRanks {
    pub tidepool: u32,
    pub wellspring: u32,
    pub quickfoot: u32,
    pub sharpened_edge: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upgrade {
    TidepoolBounty,
    ManaWellspring,
    Quickfoot,
    SharpenedEdge,
}

impl Upgrade {
    pub const ALL: &'static [Upgrade] = &[
        Upgrade::TidepoolBounty,
        Upgrade::ManaWellspring,
        Upgrade::Quickfoot,
        Upgrade::SharpenedEdge,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Upgrade::TidepoolBounty => "Tidepool's Bounty",
            Upgrade::ManaWellspring => "Mana Wellspring",
            Upgrade::Quickfoot => "Quickfoot",
            Upgrade::SharpenedEdge => "Sharpened Edge",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Upgrade::TidepoolBounty => "+2 max HP per rank.",
            Upgrade::ManaWellspring => "+1 max MP per rank.",
            Upgrade::Quickfoot => "+1 base speed per rank.",
            Upgrade::SharpenedEdge => "+20% outgoing attack damage per rank.",
        }
    }

    pub fn max_rank(&self) -> u32 {
        match self {
            Upgrade::TidepoolBounty => 10,
            Upgrade::ManaWellspring => 6,
            Upgrade::Quickfoot => 3,
            Upgrade::SharpenedEdge => 10,
        }
    }

    /// Cost in embers to advance from `current_rank` to `current_rank + 1`.
    /// Returns `None` if already at max. Curves are tuned so that maxing
    /// every ladder for a single monster costs ~75 embers — about three
    /// successful runs at ~24 embers per win.
    pub fn cost_for_next(&self, current_rank: u32) -> Option<u32> {
        if current_rank >= self.max_rank() {
            return None;
        }
        let costs: &[u32] = match self {
            Upgrade::TidepoolBounty => &[1, 1, 1, 1, 2, 2, 2, 3, 3, 3], // 19
            Upgrade::ManaWellspring => &[1, 2, 2, 2, 3, 3],             // 13
            Upgrade::Quickfoot => &[4, 5, 7],                           // 16
            Upgrade::SharpenedEdge => &[1, 2, 2, 2, 2, 3, 3, 3, 3, 5],  // 26
        };
        costs.get(current_rank as usize).copied()
    }

    pub fn current_rank(&self, ranks: &StarterRanks) -> u32 {
        match self {
            Upgrade::TidepoolBounty => ranks.tidepool,
            Upgrade::ManaWellspring => ranks.wellspring,
            Upgrade::Quickfoot => ranks.quickfoot,
            Upgrade::SharpenedEdge => ranks.sharpened_edge,
        }
    }

    fn set_rank(&self, ranks: &mut StarterRanks, rank: u32) {
        let capped = rank.min(self.max_rank());
        match self {
            Upgrade::TidepoolBounty => ranks.tidepool = capped,
            Upgrade::ManaWellspring => ranks.wellspring = capped,
            Upgrade::Quickfoot => ranks.quickfoot = capped,
            Upgrade::SharpenedEdge => ranks.sharpened_edge = capped,
        }
    }

    fn key(&self) -> &'static str {
        match self {
            Upgrade::TidepoolBounty => "tidepool",
            Upgrade::ManaWellspring => "wellspring",
            Upgrade::Quickfoot => "quickfoot",
            Upgrade::SharpenedEdge => "sharpened_edge",
        }
    }

    fn from_key(key: &str) -> Option<Upgrade> {
        match key {
            "tidepool" => Some(Upgrade::TidepoolBounty),
            "wellspring" => Some(Upgrade::ManaWellspring),
            "quickfoot" => Some(Upgrade::Quickfoot),
            "sharpened_edge" => Some(Upgrade::SharpenedEdge),
            _ => None,
        }
    }
}

static META: RwLock<Option<Meta>> = RwLock::new(None);

fn with_meta<R>(f: impl FnOnce(&Meta) -> R) -> R {
    let guard = META.read().unwrap();
    let meta = guard.as_ref().expect("meta::init not called");
    f(meta)
}

fn with_meta_mut<R>(f: impl FnOnce(&mut Meta) -> R) -> R {
    let mut guard = META.write().unwrap();
    let meta = guard.as_mut().expect("meta::init not called");
    f(meta)
}

pub fn snapshot() -> Meta {
    with_meta(|m| m.clone())
}

/// Build the canonical id string for a starter species.
pub fn starter_id(species: &str) -> MonsterId {
    format!("starter:{}", species.trim().to_lowercase())
}

/// True iff the player owns at least one monster (i.e. has gone through
/// the first-launch starter pick).
pub fn has_any_monster() -> bool {
    with_meta(|m| !m.monsters.is_empty())
}

/// Return the species name of the player's first party member (if any).
/// While the party is single-monster this drives run starter selection.
pub fn active_party_species() -> Option<String> {
    with_meta(|m| {
        m.party
            .first()
            .and_then(|id| m.monsters.get(id))
            .map(|inst| inst.species.clone())
    })
}

/// Toggle a monster's membership in the party. If it's already in the
/// party, remove it. If it's not in the party and there's room, add it.
/// Returns Ok(true) if the toggle landed, Ok(false) if it was a no-op
/// (e.g., trying to add but party is full), or Err with a hint message.
pub fn toggle_party(monster_id: &str) -> Result<bool, String> {
    let result = with_meta_mut(|meta| {
        if !meta.monsters.contains_key(monster_id) {
            return Err("That monster isn't owned.".to_string());
        }
        if let Some(idx) = meta.party.iter().position(|id| id == monster_id) {
            // Don't let the player end up with an empty party — at least
            // one member must remain to start a run.
            if meta.party.len() == 1 {
                return Err("Party can't be empty.".to_string());
            }
            meta.party.remove(idx);
            return Ok((true, meta.clone()));
        }
        if meta.party.len() >= PARTY_CAP {
            return Err(format!("Party is full (max {}).", PARTY_CAP));
        }
        meta.party.push(monster_id.to_string());
        Ok((true, meta.clone()))
    });
    match result {
        Ok((flipped, snap)) => {
            save_to_disk(&snap);
            Ok(flipped)
        }
        Err(e) => Err(e),
    }
}

/// Mint and reserve a fresh `wild:<n>` MonsterId. Increments the
/// internal counter and persists immediately.
pub fn mint_wild_id() -> MonsterId {
    let snap = with_meta_mut(|meta| {
        let id = format!("wild:{}", meta.next_wild_id);
        meta.next_wild_id = meta.next_wild_id.saturating_add(1);
        (id, meta.clone())
    });
    save_to_disk(&snap.1);
    snap.0
}

/// Push a freshly-captured monster onto the pending-purchases pile. The
/// player can buy it from the post-run shop later (independently of run
/// outcome).
pub fn push_pending_capture(instance: MonsterInstance) {
    let snap = with_meta_mut(|meta| {
        meta.pending_captures.push(instance);
        meta.clone()
    });
    save_to_disk(&snap);
}

/// Try to buy a recruit from the pending-captures pile. Returns true if
/// the purchase succeeded: embers deducted, monster moved into
/// `monsters`, and added to `party` if there's room.
pub fn try_buy_recruit(index: usize, price: u32) -> bool {
    let snap = with_meta_mut(|meta| {
        if index >= meta.pending_captures.len() {
            return None;
        }
        if meta.embers < price {
            return None;
        }
        let instance = meta.pending_captures.remove(index);
        meta.embers -= price;
        meta.monsters.insert(instance.id.clone(), instance.clone());
        // Auto-add to party when there's room. Players reorganise via
        // the Collection screen (Phase 5).
        if meta.party.len() < PARTY_CAP && !meta.party.contains(&instance.id) {
            meta.party.push(instance.id.clone());
        }
        Some(meta.clone())
    });
    let Some(snap) = snap else {
        return false;
    };
    save_to_disk(&snap);
    true
}

/// Maximum party size enforced by `try_buy_recruit`. Mirrored in the
/// capture flow so an in-run pickup of a 7th member triggers the
/// replace-slot picker.
pub const PARTY_CAP: usize = 6;

/// Register a freshly-chosen starter as owned and add it to the party.
/// No-op if it's already owned.
pub fn add_owned_starter(species: &str) {
    let id = starter_id(species);
    let snap = with_meta_mut(|meta| {
        meta.monsters.entry(id.clone()).or_insert_with(|| {
            MonsterInstance {
                id: id.clone(),
                species: species.to_string(),
            }
        });
        if !meta.party.contains(&id) {
            meta.party.push(id);
        }
        meta.clone()
    });
    save_to_disk(&snap);
}

/// Read a single monster's ranks (returns defaults if no rank has been
/// purchased for that monster yet).
pub fn ranks_for(monster_id: &str) -> StarterRanks {
    with_meta(|m| {
        m.monster_ranks
            .get(monster_id)
            .copied()
            .unwrap_or_default()
    })
}

pub fn add_embers(amount: u32) {
    let snap = with_meta_mut(|meta| {
        meta.embers = meta.embers.saturating_add(amount);
        meta.clone()
    });
    save_to_disk(&snap);
}

/// Try to purchase the next rank of an upgrade for a specific monster.
/// Returns true if the purchase succeeded.
pub fn try_buy(upgrade: Upgrade, monster_id: &str) -> bool {
    let result = with_meta_mut(|meta| {
        let ranks = meta
            .monster_ranks
            .entry(monster_id.to_string())
            .or_default();
        let current = upgrade.current_rank(ranks);
        let Some(cost) = upgrade.cost_for_next(current) else {
            return None;
        };
        if meta.embers < cost {
            return None;
        }
        meta.embers -= cost;
        upgrade.set_rank(ranks, current + 1);
        Some(meta.clone())
    });
    let Some(snap) = result else {
        return false;
    };
    save_to_disk(&snap);
    true
}

pub fn init() {
    let loaded = load_from_disk().unwrap_or_default();
    *META.write().unwrap() = Some(loaded);
}

/// Delete the meta file from disk. Used by `--reset` at startup so the
/// next `init()` loads a fresh, empty Meta.
pub fn wipe() {
    if let Some(path) = config_path() {
        let _ = std::fs::remove_file(&path);
    }
}

fn config_path() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })?;
    Some(base.join("caligo").join("meta"))
}

fn load_from_disk() -> Option<Meta> {
    let path = config_path()?;
    let contents = std::fs::read_to_string(&path).ok()?;
    let mut meta = Meta::default();
    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key == "embers" {
            meta.embers = value.parse().unwrap_or(0);
            continue;
        }
        if key == "next_wild_id" {
            meta.next_wild_id = value.parse().unwrap_or(0);
            continue;
        }
        if key == "party" {
            meta.party = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            continue;
        }
        if key == "pending_capture" {
            // Format: pending_capture=<id>|<species>
            if let Some((id, species)) = value.split_once('|') {
                meta.pending_captures.push(MonsterInstance {
                    id: id.to_string(),
                    species: species.to_string(),
                });
            }
            continue;
        }
        if let Some(rest) = key.strip_prefix("monster.") {
            // Format: monster.<id>=<species>  -> owned monster entry.
            meta.monsters.insert(
                rest.to_string(),
                MonsterInstance {
                    id: rest.to_string(),
                    species: value.to_string(),
                },
            );
            continue;
        }
        // Per-monster ranks: <MonsterId>.<upgrade>=<rank>
        if let Some((monster_id, upgrade_key)) = key.rsplit_once('.') {
            if let Some(upgrade) = Upgrade::from_key(upgrade_key) {
                let rank: u32 = value.parse().unwrap_or(0);
                let entry = meta
                    .monster_ranks
                    .entry(monster_id.to_string())
                    .or_default();
                upgrade.set_rank(entry, rank);
            }
        }
    }
    Some(meta)
}

fn save_to_disk(meta: &Meta) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut out = String::new();
    out.push_str(&format!("embers={}\n", meta.embers));
    out.push_str(&format!("next_wild_id={}\n", meta.next_wild_id));

    // Owned monsters.
    let mut monster_keys: Vec<&MonsterId> = meta.monsters.keys().collect();
    monster_keys.sort();
    for id in monster_keys {
        out.push_str(&format!("monster.{}={}\n", id, meta.monsters[id].species));
    }

    // Party assignment (single comma-separated line).
    if !meta.party.is_empty() {
        out.push_str(&format!("party={}\n", meta.party.join(",")));
    }

    // Pending captures (one per line).
    for capture in &meta.pending_captures {
        out.push_str(&format!(
            "pending_capture={}|{}\n",
            capture.id, capture.species
        ));
    }

    // Per-monster ranks. Sorted for stable diffs.
    let mut rank_keys: Vec<&MonsterId> = meta.monster_ranks.keys().collect();
    rank_keys.sort();
    for monster_id in rank_keys {
        let ranks = &meta.monster_ranks[monster_id];
        for upgrade in Upgrade::ALL {
            let rank = upgrade.current_rank(ranks);
            if rank > 0 {
                out.push_str(&format!("{}.{}={}\n", monster_id, upgrade.key(), rank));
            }
        }
    }
    let _ = std::fs::write(&path, out);
}
