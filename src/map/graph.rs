use super::node::{MapNode, NodeId};

#[derive(Debug, Clone)]
pub struct MapGraph {
    pub nodes: Vec<MapNode>,
    pub floors: Vec<Vec<NodeId>>,
    pub current: Option<NodeId>,
}

impl MapGraph {
    pub fn floor_count(&self) -> usize {
        self.floors.len()
    }

    pub fn node(&self, id: NodeId) -> &MapNode {
        &self.nodes[id]
    }

    pub fn reachable(&self) -> Vec<NodeId> {
        match self.current {
            None => self.floors.first().cloned().unwrap_or_default(),
            Some(id) => self.nodes[id].children.clone(),
        }
    }

    pub fn select(&mut self, id: NodeId) -> bool {
        if !self.reachable().contains(&id) {
            return false;
        }
        self.nodes[id].visited = true;
        self.current = Some(id);
        true
    }
}
