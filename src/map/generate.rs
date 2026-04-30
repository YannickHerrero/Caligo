use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use super::graph::MapGraph;
use super::node::{MapNode, NodeId, NodeKind};

type EdgeKey = ((u8, u8), (u8, u8));

pub const FLOORS: u8 = 14;
pub const COLUMNS: u8 = 5;
const PATHS: usize = 6;

pub fn generate() -> MapGraph {
    let mut rng = rand::thread_rng();
    generate_with(&mut rng)
}

pub fn generate_with<R: Rng>(rng: &mut R) -> MapGraph {
    let mut nodes: Vec<MapNode> = Vec::new();
    let mut floor_lists: Vec<Vec<NodeId>> = vec![Vec::new(); FLOORS as usize];
    let mut cells: HashMap<(u8, u8), NodeId> = HashMap::new();
    let mut edges: HashSet<EdgeKey> = HashSet::new();

    let boss_floor = FLOORS - 1;
    let boss = get_or_create(
        &mut nodes,
        &mut floor_lists,
        &mut cells,
        boss_floor,
        COLUMNS / 2,
    );

    let mut start_cols: Vec<u8> = Vec::with_capacity(PATHS);
    for i in 0..PATHS {
        start_cols.push((i as u8) % COLUMNS);
    }
    start_cols.shuffle(rng);

    for &start_col in &start_cols {
        let mut col = start_col;
        let mut prev = get_or_create(&mut nodes, &mut floor_lists, &mut cells, 0, col);
        for floor in 1..(FLOORS - 1) {
            let next_col = pick_next_column(floor - 1, col, &edges, rng);
            let id = get_or_create(&mut nodes, &mut floor_lists, &mut cells, floor, next_col);
            if !nodes[prev].children.contains(&id) {
                nodes[prev].children.push(id);
            }
            edges.insert(((floor - 1, col), (floor, next_col)));
            prev = id;
            col = next_col;
        }
        if !nodes[prev].children.contains(&boss) {
            nodes[prev].children.push(boss);
        }
        edges.insert(((boss_floor - 1, col), (boss_floor, COLUMNS / 2)));
    }

    let parents = compute_parents(&nodes);
    assign_kinds(&mut nodes, &floor_lists, &parents, boss, boss_floor, rng);

    for floor in floor_lists.iter_mut() {
        floor.sort_by_key(|&id| nodes[id].column);
    }

    MapGraph {
        nodes,
        floors: floor_lists,
        current: None,
    }
}

fn pick_next_column<R: Rng>(
    src_floor: u8,
    src_col: u8,
    edges: &HashSet<EdgeKey>,
    rng: &mut R,
) -> u8 {
    let mut candidates: Vec<u8> = vec![src_col];
    if src_col > 0 {
        candidates.push(src_col - 1);
    }
    if src_col < COLUMNS - 1 {
        candidates.push(src_col + 1);
    }

    // Exclude moves that would cross an existing edge. Two diagonal edges
    // between adjacent floors cross when one goes (f, c) -> (f+1, c+1)
    // and the other goes (f, c+1) -> (f+1, c).
    let dst_floor = src_floor + 1;
    let safe: Vec<u8> = candidates
        .into_iter()
        .filter(|&dst_col| {
            if dst_col == src_col {
                return true;
            }
            let crossing: EdgeKey = ((src_floor, dst_col), (dst_floor, src_col));
            !edges.contains(&crossing)
        })
        .collect();

    *safe.choose(rng).unwrap_or(&src_col)
}

fn get_or_create(
    nodes: &mut Vec<MapNode>,
    floor_lists: &mut [Vec<NodeId>],
    cells: &mut HashMap<(u8, u8), NodeId>,
    floor: u8,
    column: u8,
) -> NodeId {
    if let Some(&id) = cells.get(&(floor, column)) {
        return id;
    }
    let id = nodes.len();
    nodes.push(MapNode::new(id, NodeKind::NormalFight, floor, column));
    cells.insert((floor, column), id);
    floor_lists[floor as usize].push(id);
    id
}

fn compute_parents(nodes: &[MapNode]) -> HashMap<NodeId, Vec<NodeId>> {
    let mut parents: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in nodes {
        for &child in &node.children {
            parents.entry(child).or_default().push(node.id);
        }
    }
    parents
}

fn is_special(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Camp | NodeKind::Shop | NodeKind::EliteFight | NodeKind::Mystery
    )
}

fn banned_from_parents(
    id: NodeId,
    nodes: &[MapNode],
    parents: &HashMap<NodeId, Vec<NodeId>>,
) -> HashSet<NodeKind> {
    let mut banned = HashSet::new();
    if let Some(ps) = parents.get(&id) {
        for &p in ps {
            let k = nodes[p].kind;
            if is_special(k) {
                banned.insert(k);
            }
        }
    }
    banned
}

fn assign_kinds<R: Rng>(
    nodes: &mut [MapNode],
    floor_lists: &[Vec<NodeId>],
    parents: &HashMap<NodeId, Vec<NodeId>>,
    boss: NodeId,
    boss_floor: u8,
    rng: &mut R,
) {
    nodes[boss].kind = NodeKind::Boss;
    for &id in &floor_lists[0] {
        nodes[id].kind = NodeKind::NormalFight;
    }

    // Walk floor-by-floor so each node's parents are assigned before it,
    // letting us forbid a child from sharing a special kind with its parent.
    for floor in 1..boss_floor {
        let ids = floor_lists[floor as usize].clone();
        for id in ids {
            if id == boss {
                continue;
            }
            let kind = if floor == boss_floor - 1 {
                NodeKind::Camp
            } else {
                let banned = banned_from_parents(id, nodes, parents);
                random_kind(floor, &banned, rng)
            };
            nodes[id].kind = kind;
        }
    }
}

fn random_kind<R: Rng>(floor: u8, banned: &HashSet<NodeKind>, rng: &mut R) -> NodeKind {
    // Early floors should ease the player in: only fights and mysteries
    // until floor 3, so the first real branching choice is between
    // combat-or-curiosity and never an elite/shop/camp by surprise.
    let table: &[(NodeKind, u32)] = if floor < 3 {
        &[
            (NodeKind::EasyFight, 45),
            (NodeKind::NormalFight, 35),
            (NodeKind::Mystery, 20),
        ]
    } else if floor <= 5 {
        // Easy / Normal / Elite / Camp / Shop / Mystery
        &[
            (NodeKind::EasyFight, 35),
            (NodeKind::NormalFight, 30),
            (NodeKind::EliteFight, 4),
            (NodeKind::Camp, 12),
            (NodeKind::Shop, 8),
            (NodeKind::Mystery, 11),
        ]
    } else if floor <= 8 {
        &[
            (NodeKind::EasyFight, 22),
            (NodeKind::NormalFight, 30),
            (NodeKind::EliteFight, 8),
            (NodeKind::Camp, 12),
            (NodeKind::Shop, 10),
            (NodeKind::Mystery, 18),
        ]
    } else {
        &[
            (NodeKind::EasyFight, 12),
            (NodeKind::NormalFight, 32),
            (NodeKind::EliteFight, 14),
            (NodeKind::Camp, 12),
            (NodeKind::Shop, 12),
            (NodeKind::Mystery, 18),
        ]
    };

    weighted_pick(table, banned, rng).unwrap_or(NodeKind::NormalFight)
}

fn weighted_pick<R: Rng>(
    table: &[(NodeKind, u32)],
    banned: &HashSet<NodeKind>,
    rng: &mut R,
) -> Option<NodeKind> {
    let total: u32 = table
        .iter()
        .filter(|(k, _)| !banned.contains(k))
        .map(|(_, w)| *w)
        .sum();
    if total == 0 {
        return None;
    }
    let mut r = rng.gen_range(0..total);
    for (k, w) in table {
        if banned.contains(k) {
            continue;
        }
        if r < *w {
            return Some(*k);
        }
        r -= *w;
    }
    None
}
