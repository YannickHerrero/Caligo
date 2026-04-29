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

    assign_kinds(&mut nodes, boss, boss_floor, rng);

    for floor in floor_lists.iter_mut() {
        floor.sort_by_key(|&id| nodes[id].column);
    }

    MapGraph {
        nodes,
        floors: floor_lists,
        current: None,
        boss,
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

fn assign_kinds<R: Rng>(nodes: &mut [MapNode], boss: NodeId, boss_floor: u8, rng: &mut R) {
    for node in nodes.iter_mut() {
        node.kind = if node.id == boss {
            NodeKind::Boss
        } else if node.floor == 0 {
            NodeKind::NormalFight
        } else if node.floor == boss_floor - 1 {
            NodeKind::Camp
        } else {
            random_kind(node.floor, rng)
        };
    }
}

fn random_kind<R: Rng>(floor: u8, rng: &mut R) -> NodeKind {
    // Early floors should ease the player in: only fights and mysteries
    // until floor 3, so the first real branching choice is between
    // combat-or-curiosity and never an elite/shop/camp by surprise.
    if floor < 3 {
        let r: u32 = rng.gen_range(0..100);
        return match r {
            0..=44 => NodeKind::EasyFight,
            45..=79 => NodeKind::NormalFight,
            _ => NodeKind::Mystery,
        };
    }

    // Mid-run weights shift with depth: easy fights front-loaded, elites
    // and normal fights back-loaded. Camps and shops stay roughly steady
    // so the player always has rest and economy options available.
    let weights = if floor <= 5 {
        // Easy / Normal / Elite / Camp / Shop / Mystery
        [35, 30, 4, 12, 8, 11]
    } else if floor <= 8 {
        [22, 30, 8, 12, 10, 18]
    } else {
        [12, 32, 14, 12, 12, 18]
    };

    let total: u32 = weights.iter().sum();
    let mut r: u32 = rng.gen_range(0..total);
    let kinds = [
        NodeKind::EasyFight,
        NodeKind::NormalFight,
        NodeKind::EliteFight,
        NodeKind::Camp,
        NodeKind::Shop,
        NodeKind::Mystery,
    ];
    for (kind, w) in kinds.iter().zip(weights.iter()) {
        if r < *w {
            return *kind;
        }
        r -= *w;
    }
    NodeKind::NormalFight
}
