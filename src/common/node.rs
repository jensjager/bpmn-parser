// node.rs
use crate::common::bpmn_event::BpmnEvent;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub event: Option<BpmnEvent>,
    pub x: Option<f64>, // X-koordinaat
    pub y: Option<f64>, // Y-koordinaat
    pub pool: Option<String>, // Pool context
    pub lane: Option<String>, // Lane context
}

impl Node {
    pub fn new(id: usize, x: Option<f64>, y: Option<f64>, event: Option<BpmnEvent>, pool: Option<String>, lane: Option<String>) -> Self {
        Node { id, x, y, event, pool, lane }
    }

    pub fn with_default_event(id: usize, x: Option<f64>, y: Option<f64>) -> Self {
        Node { id, x, y, event: None, pool: None, lane: None }
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = Some(x);
        self.y = Some(y);
    }
    
    pub fn set_context(&mut self, pool: Option<String>, lane: Option<String>) {
        self.pool = pool;
        self.lane = lane;
    }
}
