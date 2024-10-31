// node.rs
use crate::common::bpmn_event::BpmnEvent;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub event: Option<BpmnEvent>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub y_offset: Option<f64>,
    pub stroke_color: Option<String>, // for example, "red", "blue", "green"
    pub fill_color: Option<String>,   // for example, "red", "blue", "green"
}


impl Node {
    pub fn new(id: usize, x: Option<f64>, y: Option<f64>, event: Option<BpmnEvent>) -> Self {
        Node { id, x, y, y_offset: Some(0.0), stroke_color: None, event, fill_color: None }
    }

    // pub fn with_default_event(id: usize, x: Option<f64>, y: Option<f64>) -> Self {
    //     Node { id, x, y, stroke_color: None, event: None, fill_color: None }
    // }

    pub fn set_position(&mut self, x: f64, y: f64, y_offset: f64) {
        self.x = Some(x);
        self.y = Some(y);
        self.y_offset = Some(y_offset);
    }
}
