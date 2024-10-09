// src/ast.rs

#[derive(Debug, Clone)]
pub enum BpmnNode {
    Event(BpmnEvent),
    Flow(BpmnFlow),
}

#[derive(Debug, Clone)]
pub enum BpmnEvent {
    Start(String),  // Start event with label
    Middle(String), // Middle event with label
    End(String),    // End event with label
}

#[derive(Debug, Clone)]
pub enum BpmnFlow {
    Nodes(Vec<BpmnNode>), // Collection of BPMN nodes
}
