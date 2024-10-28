#[derive(Debug, Clone)]
pub enum BpmnEvent {
    Start(String),              // Start event with label
    Middle(String),             // Middle event with label
    End(String),                // End event with label
    GatewayExclusive,           // Exclusive gateway event
    GatewayJoin(String),        // Join gateway event with label
    ActivityTask(String),       // Task with label

    // Recently added
    MessageEvent(String),
    TimerEvent(String),
    ConditionalEvent(String),
    SignalEvent(String),
    ErrorEvent(String),
    EscalationEvent(String),
    CompensateEvent(String),
    TerminateEvent(String),
}