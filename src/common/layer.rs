// layer.rs
use crate::common::node::Node;

pub struct Layer {
    pub layer_id: i32,
    pub nodes: Vec<Node>,
}

impl Layer {
    pub fn new(layer_id: i32, nodes: Vec<Node>) -> Self {
        Layer { layer_id, nodes }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn get_nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    pub fn get_nodes_mut(&mut self) -> &mut Vec<Node> {
        &mut self.nodes
    }

    pub fn get_layer(&self) -> i32 {
        self.layer_id
    }
}
