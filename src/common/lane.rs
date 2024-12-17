//lane.rs
use crate::common::node::Node;

pub struct Lane {
    lane: String,
    pub layers: Vec<Node>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl Lane {
    pub fn new(lane: String) -> Self {
        Lane {
            lane,
            layers: Vec::new(),
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.layers.push(node);
    }

    pub fn get_layers(&self) -> &Vec<Node> {
        &self.layers
    }

    pub fn get_layers_mut(&mut self) -> &mut Vec<Node> {
        &mut self.layers
    }

    pub fn get_lane(&self) -> &String {
        &self.lane
    }

    pub fn sort_nodes_by_layer_id(&mut self) {
        self.layers.sort_by(|a, b| a.layer_id.cmp(&b.layer_id));
    }

    pub fn get_nodes_by_layer_id_mut(&mut self, layer_id: usize) -> Vec<&mut Node> {
        self.layers
            .iter_mut()
            .filter(|node| node.layer_id.unwrap_or(0) == layer_id)
            .collect()
    }

    pub fn get_node_by_id(&self, id: usize) -> Option<&Node> {
        self.layers.iter().find(|node| node.id == id)
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height);
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = Some(x);
        self.y = Some(y);
    }
}
