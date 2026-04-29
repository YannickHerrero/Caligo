use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

use super::graph::MapGraph;
use super::node::{MapNode, NodeId, NodeKind};

const FLOORS: u8 = 14;
const COLUMNS: u8 = 6;
const PATHS: usize = 6;

pub fn generate() -> MapGraph {
    let mut rng = rand::thread_rng();
    generate_with(&mut rng)
}

pub fn generate_with<R: Rng>(rng: &mut R) -> MapGraph {
    let mut nodes: Vec<MapNode> = Vec::new();
    let mut floor_lists: Vec<Vec<NodeId>> = vec![Vec::new(); FLOORS as usize];
    let mut cells: HashMap<(u8, u8), NodeId> = HashMap::new();

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
            col = pick_next_column(col, rng);
            let id = get_or_create(&mut nodes, &mut floor_lists, &mut cells, floor, col);
            if !nodes[prev].children.contains(&id) {
                nodes[prev].children.push(id);
            }
            prev = id;
        }
        if !nodes[prev].children.contains(&boss) {
            nodes[prev].children.push(boss);
        }
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

fn pick_next_column<R: Rng>(col: u8, rng: &mut R) -> u8 {
    let mut choices: Vec<u8> = vec![col];
    if col > 0 {
        choices.push(col - 1);
    }
    if col < COLUMNS - 1 {
        choices.push(col + 1);
    }
    *choices.choose(rng).unwrap_or(&col)
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
