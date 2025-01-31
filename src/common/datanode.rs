use crate::common::graph::DataNodeId;

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
}

impl DataNode {
    pub fn set_position(&mut self, x: f64, y: f64, x_offset: f64, y_offset: f64) {
        self.x = Some(x);
        self.y = Some(y);
        self.x_offset = Some(x_offset);
        self.y_offset = Some(y_offset);
    }

    pub fn set_layer_id(&mut self, layer_id: usize) {
        self.layer_id = Some(layer_id);
    }
}
