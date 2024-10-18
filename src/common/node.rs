// node.rs
use crate::common::bpmn_event::BpmnEvent;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub event: Option<BpmnEvent>,
    pub layer: Option<i32>,
}

impl Node {
    pub fn new(id: usize, layer: Option<i32>, event: Option<BpmnEvent>) -> Self {
        Node { id, layer, event }
    }

    pub fn with_default_event(id: usize, layer: Option<i32>) -> Self {
        Node { id, layer, event: None }
    }
}
