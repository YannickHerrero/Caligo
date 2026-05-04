//! Cross-run meta state: embers (the meta-currency) and per-starter
//! ranks in permanent stat upgrades. Persists to a small file alongside
//! the settings.
//!
//! Ranks are stored per starter (and, in the future, per party member)
//! rather than globally — buying a rank for Pinchy doesn't affect Cinder
//! or Sprout. The on-disk format uses dotted keys (e.g.
//! `pinchy.tidepool=3`) so adding more upgrades or characters is purely
//! additive.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Meta {
    /// Total unspent meta-currency.
    pub embers: u32,
    /// Permanent ranks per starter (key = starter.name lowercased).
    pub starter_ranks: HashMap<String, StarterRanks>,
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
    /// Returns `None` if already at max.
    pub fn cost_for_next(&self, current_rank: u32) -> Option<u32> {
        if current_rank >= self.max_rank() {
            return None;
        }
        let costs: &[u32] = match self {
            Upgrade::TidepoolBounty => &[1, 2, 3, 4, 6, 8, 10, 14, 19, 25],
            Upgrade::ManaWellspring => &[1, 2, 3, 6, 10, 15],
            Upgrade::Quickfoot => &[7, 15, 25],
            Upgrade::SharpenedEdge => &[2, 4, 6, 8, 12, 16, 22, 30, 40, 55],
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

pub fn snapshot() -> Meta {
    with_meta(|m| m.clone())
}

/// Read a single starter's ranks (returns defaults if the starter has
/// never had a rank purchased).
pub fn ranks_for(starter: &str) -> StarterRanks {
    let key = normalize_starter(starter);
    with_meta(|m| m.starter_ranks.get(&key).copied().unwrap_or_default())
}

pub fn add_embers(amount: u32) {
    let snap = {
        let mut guard = META.write().unwrap();
        let meta = guard.as_mut().expect("meta::init not called");
        meta.embers = meta.embers.saturating_add(amount);
        meta.clone()
    };
    save_to_disk(&snap);
}

/// Try to purchase the next rank of an upgrade for a specific starter.
/// Returns true if the purchase succeeded.
pub fn try_buy(upgrade: Upgrade, starter: &str) -> bool {
    let key = normalize_starter(starter);
    let snap = {
        let mut guard = META.write().unwrap();
        let meta = guard.as_mut().expect("meta::init not called");
        let ranks = meta.starter_ranks.entry(key).or_default();
        let current = upgrade.current_rank(ranks);
        let Some(cost) = upgrade.cost_for_next(current) else {
            return false;
        };
        if meta.embers < cost {
            return false;
        }
        meta.embers -= cost;
        upgrade.set_rank(ranks, current + 1);
        meta.clone()
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

fn normalize_starter(name: &str) -> String {
    name.trim().to_lowercase()
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
        let parsed: u32 = value.parse().unwrap_or(0);
        if key == "embers" {
            meta.embers = parsed;
            continue;
        }
        // Per-starter ranks use dotted keys: `pinchy.tidepool=3`.
        if let Some((starter, upgrade_key)) = key.split_once('.') {
            if let Some(upgrade) = Upgrade::from_key(upgrade_key) {
                let entry = meta
                    .starter_ranks
                    .entry(starter.to_string())
                    .or_default();
                upgrade.set_rank(entry, parsed);
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
    let mut keys: Vec<&String> = meta.starter_ranks.keys().collect();
    keys.sort();
    for starter in keys {
        let ranks = &meta.starter_ranks[starter];
        for upgrade in Upgrade::ALL {
            let rank = upgrade.current_rank(ranks);
            if rank > 0 {
                out.push_str(&format!("{}.{}={}\n", starter, upgrade.key(), rank));
            }
        }
    }
    let _ = std::fs::write(&path, out);
}
