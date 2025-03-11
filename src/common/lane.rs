//lane.rs
use crate::common::graph::NodeId;
use crate::common::node::Node;

#[derive(Default)]
pub struct Lane {
    pub lane: Option<String>,
    /// These will be sorted by layers at some point.
    pub nodes: Vec<NodeId>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl Lane {
    pub fn new(lane: Option<String>) -> Self {
        Lane {
            lane,
            ..Default::default()
        }
    }

    pub fn sort_nodes_by_layer_id(&mut self, nodes: &[Node]) {
        self.nodes
            .sort_by(|a, b| nodes[a.0].layer_id.cmp(&nodes[b.0].layer_id));
    }
}
