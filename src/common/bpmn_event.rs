use crate::lexer::DataMeta;
use crate::lexer::EventMeta;
use crate::lexer::GatewayType;
use crate::lexer::NodeMeta;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BpmnEvent {
    Start(EventMeta),                              // Start event with label
    Middle(EventMeta),                             // Middle event with label
    End(EventMeta),                                // End event with label
    Gateway(GatewayType),                          // Exclusive gateway event
    GatewayJoin(String),                           // Join gateway event with label
    ActivityTask(NodeMeta),                        // Task with label
    ActivitySubprocess(NodeMeta),                  // Subprocess with label
    ActivityCallActivity(NodeMeta),                // Call Activity with label
    ActivityEventSubprocess(NodeMeta),             // Event Subprocess with label
    ActivityTransaction(NodeMeta),                 // Transaction with label
    StartTimerEvent(EventMeta),                    // Timer start event with label
    StartSignalEvent(EventMeta),                   // Signal start event with label
    StartMessageEvent(EventMeta),                  // Message start event with label
    StartConditionalEvent(EventMeta),              // Conditional start event with label
    EndErrorEvent(EventMeta),                      // Error end event with label
    EndCancelEvent(EventMeta),                     // Cancel end event with label
    EndSignalEvent(EventMeta),                     // Signal end event with label
    EndMessageEvent(EventMeta),                    // Message end event with label
    EndTerminateEvent(EventMeta),                  // Terminate end event with label
    EndEscalationEvent(EventMeta),                 // Escalation end event with label
    EndCompensationEvent(EventMeta),               // Compensation end event with label
    BoundaryEvent(String, usize, bool), // Boundary event with label, attached to node ID, cancel activity flag
    BoundaryErrorEvent(String, usize, bool), // Error boundary event
    BoundaryTimerEvent(String, usize, bool), // Timer boundary event
    BoundaryCancelEvent(String, usize, bool), // Cancel boundary event
    BoundarySignalEvent(String, usize, bool), // Signal boundary event
    BoundaryMessageEvent(String, usize, bool), // Message boundary event
    BoundaryEscalationEvent(String, usize, bool), // Escalation boundary event
    BoundaryConditionalEvent(String, usize, bool), // Conditional boundary event
    BoundaryCompensationEvent(String, usize), // Compensation boundary event (always non-interrupting)
    DataStoreReference(DataMeta),             // Data store reference with label
    DataObjectReference(DataMeta),            // Data object reference with label
    TaskUser(NodeMeta),                       // User task with label
    TaskService(NodeMeta),                    // Service task with label
    TaskBusinessRule(NodeMeta),               // Business rule task with label
    TaskScript(NodeMeta),                     // Script task with label
    Dummy(),
}

pub fn get_node_size(event: &BpmnEvent) -> (usize, usize) {
    match event {
        // Start Events
        BpmnEvent::Start(_)
        | BpmnEvent::StartTimerEvent(_)
        | BpmnEvent::StartSignalEvent(_)
        | BpmnEvent::StartMessageEvent(_)
        | BpmnEvent::StartConditionalEvent(_) => (36, 36),

        BpmnEvent::Middle(_) => (36, 36),

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
        BpmnEvent::Gateway(_) => (50, 50),

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

        // BpmnEvent::Dummy() => (10, 10),

        // Default case for any other elements
        _ => (100, 80),
    }
}
