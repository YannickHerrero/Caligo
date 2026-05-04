//! Cross-run meta state: embers (the meta-currency) and permanent stat
//! ladder ranks earned from spending them. Persists to a small file
//! alongside the settings.

use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Meta {
    /// Total unspent meta-currency.
    pub embers: u32,
    /// Permanent +max HP rank (0..=Upgrade::TidepoolBounty.max_rank()).
    pub tidepool_rank: u32,
    /// Permanent +max MP rank.
    pub wellspring_rank: u32,
    /// Permanent +speed rank.
    pub quickfoot_rank: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upgrade {
    TidepoolBounty,
    ManaWellspring,
    Quickfoot,
}

impl Upgrade {
    pub const ALL: &'static [Upgrade] = &[
        Upgrade::TidepoolBounty,
        Upgrade::ManaWellspring,
        Upgrade::Quickfoot,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Upgrade::TidepoolBounty => "Tidepool's Bounty",
            Upgrade::ManaWellspring => "Mana Wellspring",
            Upgrade::Quickfoot => "Quickfoot",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Upgrade::TidepoolBounty => "+2 max HP per rank.",
            Upgrade::ManaWellspring => "+1 max MP per rank.",
            Upgrade::Quickfoot => "+1 base speed per rank.",
        }
    }

    pub fn max_rank(&self) -> u32 {
        match self {
            Upgrade::TidepoolBounty => 10,
            Upgrade::ManaWellspring => 6,
            Upgrade::Quickfoot => 3,
        }
    }

    /// Cost in embers to advance from `current_rank` to `current_rank + 1`.
    /// Returns `None` if already at max.
    pub fn cost_for_next(&self, current_rank: u32) -> Option<u32> {
        if current_rank >= self.max_rank() {
            return None;
        }
        let costs: &[u32] = match self {
            Upgrade::TidepoolBounty => &[5, 8, 12, 16, 22, 30, 40, 55, 75, 100],
            Upgrade::ManaWellspring => &[5, 10, 15, 25, 40, 60],
            Upgrade::Quickfoot => &[30, 60, 100],
        };
        costs.get(current_rank as usize).copied()
    }

    pub fn current_rank(&self, meta: &Meta) -> u32 {
        match self {
            Upgrade::TidepoolBounty => meta.tidepool_rank,
            Upgrade::ManaWellspring => meta.wellspring_rank,
            Upgrade::Quickfoot => meta.quickfoot_rank,
        }
    }

    fn set_rank(&self, meta: &mut Meta, rank: u32) {
        let capped = rank.min(self.max_rank());
        match self {
            Upgrade::TidepoolBounty => meta.tidepool_rank = capped,
            Upgrade::ManaWellspring => meta.wellspring_rank = capped,
            Upgrade::Quickfoot => meta.quickfoot_rank = capped,
        }
    }
}

static META: RwLock<Meta> = RwLock::new(Meta {
    embers: 0,
    tidepool_rank: 0,
    wellspring_rank: 0,
    quickfoot_rank: 0,
});

pub fn snapshot() -> Meta {
    *META.read().unwrap()
}

pub fn add_embers(amount: u32) {
    let snap = {
        let mut m = META.write().unwrap();
        m.embers = m.embers.saturating_add(amount);
        *m
    };
    save_to_disk(&snap);
}

/// Try to purchase the next rank of an upgrade. Returns true if the
/// purchase succeeded (rank advanced, embers deducted, written to disk).
pub fn try_buy(upgrade: Upgrade) -> bool {
    let snap = {
        let mut m = META.write().unwrap();
        let current = upgrade.current_rank(&m);
        let Some(cost) = upgrade.cost_for_next(current) else {
            return false;
        };
        if m.embers < cost {
            return false;
        }
        m.embers -= cost;
        upgrade.set_rank(&mut m, current + 1);
        *m
    };
    save_to_disk(&snap);
    true
}

pub fn init() {
    if let Some(loaded) = load_from_disk() {
        *META.write().unwrap() = loaded;
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
        let value = value.trim();
        let parsed: u32 = value.parse().unwrap_or(0);
        match key.trim() {
            "embers" => meta.embers = parsed,
            "tidepool_rank" => meta.tidepool_rank = parsed,
            "wellspring_rank" => meta.wellspring_rank = parsed,
            "quickfoot_rank" => meta.quickfoot_rank = parsed,
            _ => {}
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
    let contents = format!(
        "embers={}\ntidepool_rank={}\nwellspring_rank={}\nquickfoot_rank={}\n",
        meta.embers, meta.tidepool_rank, meta.wellspring_rank, meta.quickfoot_rank,
    );
    let _ = std::fs::write(&path, contents);
}
