#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BpmnEvent {
    Start(String),              // Start event with label
    Middle(String),             // Middle event with label
    End(String),                // End event with label
    GatewayExclusive,           // Exclusive gateway event
    GatewayInclusive,           // Inclusive gateway event
    GatewayParallel,            // Parallel gateway event
    GatewayEvent,               // Event-based gateway event
    GatewayJoin(String),        // Join gateway event with label
    ActivityTask(String),       // Task with label
    ActivitySubprocess(String),         // Subprocess with label
    ActivityCallActivity(String),       // Call Activity with label
    ActivityEventSubprocess(String),    // Event Subprocess with label
    ActivityTransaction(String),        // Transaction with label
    StartTimerEvent(String),            // Timer start event with label
    StartSignalEvent(String),           // Signal start event with label
    StartMessageEvent(String),          // Message start event with label
    StartConditionalEvent(String),      // Conditional start event with label
    EndErrorEvent(String),              // Error end event with label
    EndCancelEvent(String),             // Cancel end event with label
    EndSignalEvent(String),             // Signal end event with label
    EndMessageEvent(String),            // Message end event with label
    EndTerminateEvent(String),          // Terminate end event with label
    EndEscalationEvent(String),         // Escalation end event with label
    EndCompensationEvent(String),       // Compensation end event with label
    BoundaryEvent(String, usize, bool),             // Boundary event with label, attached to node ID, cancel activity flag
    BoundaryErrorEvent(String, usize, bool),        // Error boundary event
    BoundaryTimerEvent(String, usize, bool),        // Timer boundary event
    BoundaryCancelEvent(String, usize, bool),       // Cancel boundary event
    BoundarySignalEvent(String, usize, bool),       // Signal boundary event
    BoundaryMessageEvent(String, usize, bool),      // Message boundary event
    BoundaryEscalationEvent(String, usize, bool),   // Escalation boundary event
    BoundaryConditionalEvent(String, usize, bool),  // Conditional boundary event
    BoundaryCompensationEvent(String, usize),       // Compensation boundary event (always non-interrupting)
    DataStoreReference(String),         // Data store reference with label
    DataObjectReference(String),        // Data object reference with label
    TaskUser(String),                   // User task with label
    TaskService(String),                // Service task with label
    TaskBusinessRule(String),           // Business rule task with label
    TaskScript(String),                 // Script task with label
}

pub fn get_node_size(event: &BpmnEvent) -> (usize, usize) {
    match event {
        // Start Events
        BpmnEvent::Start(_)
        | BpmnEvent::StartTimerEvent(_)
        | BpmnEvent::StartSignalEvent(_)
        | BpmnEvent::StartMessageEvent(_)
        | BpmnEvent::StartConditionalEvent(_) => (36, 36),

        // End Events
        BpmnEvent::End(_)
        | BpmnEvent::EndErrorEvent(_)
        | BpmnEvent::EndCancelEvent(_)
        | BpmnEvent::EndSignalEvent(_)
        | BpmnEvent::EndMessageEvent(_)
        | BpmnEvent::EndTerminateEvent(_)
        | BpmnEvent::EndEscalationEvent(_)
        | BpmnEvent::EndCompensationEvent(_) => (36, 36),

        // Gateways
        BpmnEvent::GatewayExclusive
        | BpmnEvent::GatewayInclusive
        | BpmnEvent::GatewayParallel
        | BpmnEvent::GatewayEvent
        | BpmnEvent::GatewayJoin(_) => (50, 50),

        // Activities
        BpmnEvent::ActivityTask(_)
        | BpmnEvent::ActivityCallActivity(_)
        | BpmnEvent::TaskUser(_)
        | BpmnEvent::TaskService(_)
        | BpmnEvent::TaskBusinessRule(_)
        | BpmnEvent::TaskScript(_) => (100, 80),

        // Subprocesses and Transactions (expanded)
        BpmnEvent::ActivitySubprocess(_)
        | BpmnEvent::ActivityEventSubprocess(_)
        | BpmnEvent::ActivityTransaction(_) => (350, 200),

        // Boundary Events
        BpmnEvent::BoundaryEvent(_, _, _)
        | BpmnEvent::BoundaryErrorEvent(_, _, _)
        | BpmnEvent::BoundaryTimerEvent(_, _, _)
        | BpmnEvent::BoundaryCancelEvent(_, _, _)
        | BpmnEvent::BoundarySignalEvent(_, _, _)
        | BpmnEvent::BoundaryMessageEvent(_, _, _)
        | BpmnEvent::BoundaryEscalationEvent(_, _, _)
        | BpmnEvent::BoundaryConditionalEvent(_, _, _)
        | BpmnEvent::BoundaryCompensationEvent(_, _) => (36, 36),

        // Data Objects
        BpmnEvent::DataStoreReference(_) => (50, 50),
        BpmnEvent::DataObjectReference(_) => (36, 50),

        // Default case for any other elements
        _ => (100, 80),
    }
}