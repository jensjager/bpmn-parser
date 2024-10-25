// node.rs
use crate::common::bpmn_event::BpmnEvent;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub event: Option<BpmnEvent>,
    pub x: Option<f64>, // X-koordinaat
    pub y: Option<f64>, // Y-koordinaat
}

impl Node {
    pub fn new(id: usize, x: Option<f64>, y: Option<f64>, event: Option<BpmnEvent>) -> Self {
        Node { id, x, y, event }
    }

    pub fn with_default_event(id: usize, x: Option<f64>, y: Option<f64>) -> Self {
        Node { id, x, y, event: None }
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = Some(x);
        self.y = Some(y);
    }
}
