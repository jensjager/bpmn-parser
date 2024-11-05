#[derive(Debug, Clone)]
pub enum BpmnEvent {
    Start(String),              // Start event with label
    Middle(String),             // Middle event with label
    End(String),                // End event with label
    GatewayExclusive,           // Exclusive gateway event
    GatewayJoin(String),        // Join gateway event with label
    ActivityTask(String),       // Task with label
}

pub fn get_node_size(event: &BpmnEvent) -> (usize, usize) {
    match event {
        BpmnEvent::Start(_) => (36, 36),
        BpmnEvent::End(_) => (36, 36),
        BpmnEvent::Middle(_) => (100, 80),
        BpmnEvent::ActivityTask(_) => (100, 80),
        BpmnEvent::GatewayExclusive => (50, 50),
        BpmnEvent::GatewayJoin(_) => (50, 50),
    }
}