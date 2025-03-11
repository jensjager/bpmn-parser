// node.rs
use crate::common::bpmn_event::*;
use crate::common::graph::EdgeId;
use crate::common::graph::NodeId;

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub id: NodeId,
    pub event: Option<BpmnEvent>,
    pub x: Option<f64>,
    pub x_offset: Option<f64>,
    pub y: Option<f64>,
    pub y_offset: Option<f64>,
    pub stroke_color: Option<String>,
    pub fill_color: Option<String>,
    pub pool: Option<String>,
    pub lane: Option<String>,
    pub layer_id: Option<usize>,
    pub pos_in_layer: Option<usize>,

    // TODO is this actually really used? Was set in solve_layer_assignment but
    // not read anywhere.
    //pub crosses_lanes: bool,
    //pub to_node_id: Option<NodeId>,

    // TODO do sequence flow edges, message edges and data object edges be stored differently?
    pub incoming: Vec<EdgeId>,
    pub outgoing: Vec<EdgeId>,
}

impl Node {}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node {{ id: {}, x: {:?}, y: {:?}, event: {:?}, pool: {:?}, lane: {:?} }}",
            self.id, self.x, self.y, self.event, self.pool, self.lane
        )
    }
}
