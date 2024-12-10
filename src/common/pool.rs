// pool.rs
use crate::common::lane::Lane;
use crate::common::node::Node;
use std::collections::HashMap;
pub struct Pool {
    pool_name: String,
    lanes: Vec<Lane>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl Pool {
    pub fn new(pool_name: String) -> Self {
        Pool {
            pool_name,
            lanes: Vec::new(),
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        let node_lane = node
            .lane
            .clone()
            .unwrap_or_else(|| "default_lane".to_string());
        if let Some(lane) = self.lanes.iter_mut().find(|l| l.get_lane() == &node_lane) {
            lane.add_node(node);
        } else {
            let mut new_lane = Lane::new(node_lane);
            new_lane.add_node(node);
            self.lanes.push(new_lane);
        }
    }

    pub fn add_lane(&mut self, lane: Lane) {
        self.lanes.push(lane);
    }

    pub fn get_lanes(&self) -> &Vec<Lane> {
        &self.lanes
    }

    pub fn get_lanes_mut(&mut self) -> &mut Vec<Lane> {
        &mut self.lanes
    }

    pub fn get_pool_name(&self) -> String {
        self.pool_name.clone()
    }

    pub fn get_node_by_id(&self, id: usize) -> Option<&Node> {
        for lane in &self.lanes {
            if let Some(node) = lane.get_node_by_id(id) {
                return Some(node);
            }
        }
        None
    }

    pub fn get_nodes_by_id(&self) -> HashMap<usize, Node> {
        let mut nodes_by_id = HashMap::new();
        for lane in &self.lanes {
            for node in lane.get_layers() {
                nodes_by_id.insert(node.id, node.clone());
            }
        }
        nodes_by_id
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height);
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }
}
