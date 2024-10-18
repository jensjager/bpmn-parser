#[derive(Debug, Clone)]
pub enum BpmnEvent {
    Start(String),              // Start event with label
    Middle(String),             // Middle event with label
    End(String),                // End event with label
    GatewayExclusive,           // Exclusive gateway event
    GatewayJoin(String),        // Join gateway event with label
    ActivityTask(String),       // Task with label
}