// node.rs
use crate::common::bpmn_event::*;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub event: Option<BpmnEvent>,
    pub x: Option<f64>,
    pub x_offset: Option<f64>,
    pub y: Option<f64>,
    pub y_offset: Option<f64>,
    pub pool: Option<String>,
    pub lane: Option<String>,
    pub layer_id: Option<usize>,
    pub crosses_lanes: bool,
    pub to_node_id: Option<usize>,
}

impl Node {
    pub fn new(
        id: usize,
        x: Option<f64>,
        y: Option<f64>,
        event: Option<BpmnEvent>,
        pool: Option<String>,
        lane: Option<String>,
    ) -> Self {
        Node {
            id,
            x,
            x_offset: Some(0.0),
            y,
            y_offset: Some(0.0),
            event,
            pool,
            lane,
            layer_id: None,
            crosses_lanes: false,
            to_node_id: None,
        }
    }

    pub fn set_position(&mut self, x: f64, y: f64, x_offset: f64, y_offset: f64) {
        self.x = Some(x);
        self.y = Some(y);
        self.x_offset = Some(x_offset);
        self.y_offset = Some(y_offset);
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node {{ id: {}, x: {:?}, y: {:?}, event: {:?}, pool: {:?}, lane: {:?} }}",
            self.id, self.x, self.y, self.event, self.pool, self.lane
        )
    }
}
