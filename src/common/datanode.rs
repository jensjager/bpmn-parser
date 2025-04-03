use crate::common::graph::{DataEdgeId, DataNodeId, NodeId};

use super::bpmn_event::BpmnEvent;

#[derive(Debug, Clone, Default)]
pub struct DataNode {
    pub id: DataNodeId,
    pub datatype: Option<BpmnEvent>,
    pub x: Option<f64>,
    pub x_offset: Option<f64>,
    pub y: Option<f64>,
    pub y_offset: Option<f64>,
    pub pool: Option<String>,
    pub lane: Option<String>,
    pub layer_id: Option<usize>,
    pub uses_half_layer: bool,
    pub reference_node: Option<NodeId>,
    pub pos_in_layer: Option<usize>,
    pub incoming: Vec<DataEdgeId>,
    pub outgoing: Vec<DataEdgeId>,
}

impl DataNode {}

impl std::fmt::Display for DataNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node {{ id: {}, x: {:?}, y: {:?}, event: {:?}, pool: {:?}, lane: {:?} }}",
            self.id, self.x, self.y, self.datatype, self.pool, self.lane
        )
    }
}
