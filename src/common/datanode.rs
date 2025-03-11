use crate::common::graph::{DataNodeId, NodeId};

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
    pub above: bool,
    pub reference_node: Option<NodeId>,
    pub pos_in_layer: Option<usize>,
}

impl DataNode {
    pub fn set_position(&mut self, x: f64, y: f64, x_offset: f64, y_offset: f64) {
        self.x = Some(x);
        if self.above {
            self.y = Some(y - 80.0);
        } else {
            self.y = Some(y + 150.0);
        }

        self.x_offset = Some(x_offset);
        self.y_offset = Some(y_offset);
    }
}

impl std::fmt::Display for DataNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node {{ id: {}, x: {:?}, y: {:?}, event: {:?}, pool: {:?}, lane: {:?} }}",
            self.id, self.x, self.y, self.datatype, self.pool, self.lane
        )
    }
}
