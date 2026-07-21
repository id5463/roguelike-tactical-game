use serde::{Serialize, Deserialize};
use crate::rng::SeededRng;
use crate::types::NodeId;

/// Node type on the overworld map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Campfire,
    Shop,
    Monster,
    Event,
    Boss,
    Start,
}

impl NodeType {
    pub fn as_char(&self) -> char {
        match self {
            NodeType::Campfire => 'C',
            NodeType::Shop => '$',
            NodeType::Monster => 'M',
            NodeType::Event => 'E',
            NodeType::Boss => 'B',
            NodeType::Start => 'S',
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Campfire => "Campfire",
            NodeType::Shop => "Shop",
            NodeType::Monster => "Monster",
            NodeType::Event => "Event",
            NodeType::Boss => "BOSS",
            NodeType::Start => "Start",
        }
    }
}

/// A node on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub connections: Vec<NodeId>,
    pub visited: bool,
    pub cleared: bool,
}

/// A single floor (10 nodes + boss).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorMap {
    pub floor_num: u32,
    pub columns: Vec<Vec<MapNode>>,
    pub current_column: usize,
    pub current_node: Option<NodeId>,
}

impl FloorMap {
    pub fn generate(floor_num: u32, rng: &mut SeededRng) -> Self {
        // 12 columns: start, 10 node columns, boss
        let mut columns: Vec<Vec<MapNode>> = Vec::new();

        // Column 0: start
        columns.push(vec![MapNode {
            id: NodeId { floor: floor_num, index: 0 },
            node_type: NodeType::Start,
            connections: Vec::new(),
            visited: true,
            cleared: true,
        }]);

        // Columns 1..=10: regular nodes
        for col in 1..=10u32 {
            let num_nodes = rng.gen_range(2, 4) as usize;
            let mut nodes = Vec::new();
            for row in 0..num_nodes {
                let idx = col * 10 + row as u32;
                let node_type = if col == 10 {
                    match rng.gen_range(0, 2) {
                        0 => NodeType::Monster,
                        _ => NodeType::Event,
                    }
                } else {
                    match rng.gen_range(0, 9) {
                        0..=4 => NodeType::Monster,
                        5 => NodeType::Campfire,
                        6 => NodeType::Shop,
                        _ => NodeType::Event,
                    }
                };
                nodes.push(MapNode {
                    id: NodeId { floor: floor_num, index: idx },
                    node_type,
                    connections: Vec::new(),
                    visited: false,
                    cleared: false,
                });
            }
            columns.push(nodes);
        }

        // Last column: boss
        let boss_idx = 999;
        columns.push(vec![MapNode {
            id: NodeId { floor: floor_num, index: boss_idx },
            node_type: NodeType::Boss,
            connections: Vec::new(),
            visited: false,
            cleared: false,
        }]);

        // Connect columns
        for col in 0..columns.len() - 1 {
            let num_cur = columns[col].len();
            let num_next = columns[col + 1].len();
            for i in 0..num_cur {
                let num_conns = (rng.gen_range(1, 3.min(num_next as i32))) as usize;
                let mut targets: Vec<usize> = Vec::new();
                let base = (i as f64 / num_cur as f64 * num_next as f64).floor() as i32;
                for _ in 0..num_conns {
                    let offset = rng.gen_range(-1, 1);
                    let mut t = (base + offset).clamp(0, num_next as i32 - 1) as usize;
                    let mut tries = 0;
                    while targets.contains(&t) && tries < 5 {
                        t = (t + 1) % num_next;
                        tries += 1;
                    }
                    if !targets.contains(&t) {
                        targets.push(t);
                    }
                }
                for &t in &targets {
                    let next_id = columns[col + 1][t].id;
                    columns[col][i].connections.push(next_id);
                }
            }
        }

        Self {
            floor_num,
            columns,
            current_column: 0,
            current_node: Some(NodeId { floor: floor_num, index: 0 }),
        }
    }

    pub fn available_nodes(&self) -> Vec<&MapNode> {
        if self.current_column >= self.columns.len() - 1 {
            return Vec::new();
        }
        if let Some(cur_id) = self.current_node {
            let cur_col = self.columns.get(self.current_column).unwrap();
            if let Some(cur_node) = cur_col.iter().find(|n| n.id == cur_id) {
                let mut result = Vec::new();
                for conn in &cur_node.connections {
                    if let Some(next_col) = self.columns.get(self.current_column + 1) {
                        if let Some(node) = next_col.iter().find(|n| n.id == *conn) {
                            result.push(node);
                        }
                    }
                }
                return result;
            }
        }
        Vec::new()
    }

    pub fn visit_node(&mut self, node_id: NodeId) -> bool {
        // Find which column this node belongs to first
        let target_col = self.columns.iter().enumerate()
            .find(|(_, col)| col.iter().any(|n| n.id == node_id))
            .map(|(ci, _)| ci);
        if let Some(ci) = target_col {
            for col in &mut self.columns {
                for node in col.iter_mut() {
                    if node.id == node_id {
                        node.visited = true;
                        node.cleared = true;
                        self.current_node = Some(node_id);
                        self.current_column = ci;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn render_tree(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("=== Floor {} Map ===\n", self.floor_num + 1));
        for (ci, col) in self.columns.iter().enumerate() {
            out.push_str(&format!("Col {:2}: ", ci));
            for node in col {
                let marker = if node.cleared { 'x' }
                             else if node.visited { 'o' }
                             else { node.node_type.as_char() };
                let available = self.available_nodes().iter().any(|n| n.id == node.id);
                if available {
                    out.push_str(&format!("[{}]({}) ", marker, node.id.index));
                } else {
                    out.push_str(&format!(" {}({}) ", marker, node.id.index));
                }
            }
            out.push('\n');
        }
        out
    }
}

/// Overworld state across all floors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overworld {
    pub floors: Vec<FloorMap>,
    pub current_floor: u32,
    pub total_floors: u32,
    pub gold: i32,
    pub score_damage_dealt: i64,
}

impl Overworld {
    pub fn new(total_floors: u32, rng: &mut SeededRng) -> Self {
        let mut floors = Vec::new();
        for f in 0..total_floors {
            floors.push(FloorMap::generate(f, rng));
        }
        Self {
            floors,
            current_floor: 0,
            total_floors,
            gold: 100,
            score_damage_dealt: 0,
        }
    }

    pub fn current_floor_map(&self) -> &FloorMap {
        &self.floors[self.current_floor as usize]
    }

    pub fn current_floor_map_mut(&mut self) -> &mut FloorMap {
        &mut self.floors[self.current_floor as usize]
    }

    pub fn advance_floor(&mut self) -> bool {
        if self.current_floor + 1 < self.total_floors {
            self.current_floor += 1;
            true
        } else {
            false
        }
    }
}
