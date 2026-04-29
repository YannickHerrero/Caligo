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
            random_kind(rng)
        };
    }
}

fn random_kind<R: Rng>(rng: &mut R) -> NodeKind {
    let r: u32 = rng.gen_range(0..100);
    match r {
        0..=24 => NodeKind::EasyFight,
        25..=54 => NodeKind::NormalFight,
        55..=62 => NodeKind::EliteFight,
        63..=74 => NodeKind::Camp,
        75..=84 => NodeKind::Shop,
        _ => NodeKind::Mystery,
    }
}
