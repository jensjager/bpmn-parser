use std::fs;
use std::process::Command;

use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::Graph;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::to_xml::generate_bpmn;

#[test]
fn test_uc1_define_bpmn_elements() {
    // UC1: Define BPMN elements using a human-readable language

    // Example DSL input defining various BPMN elements
    let input = r#"
# Start Event
- Task A
X
->Branch1 "Condition 1"
Branch1:
- Task B1
J JoinPoint "Joining Branches"
X->Branch2 "Condition 2"
Branch2:
- Task B2
J JoinPoint "Joining Branches"
X<-JoinPoint
- Task C
. End Event
"#;

    // Initialize the lexer and parser
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    // Parse the input
    let parse_result = parser.parse();
    assert!(
        parse_result.is_ok(),
        "Parser should successfully process the UC1 input"
    );
    let graph = parse_result.unwrap();

    // Verify that BPMN elements have been defined and added to the graph
    assert!(!graph.nodes.is_empty(), "Graph should contain nodes");
    assert!(!graph.edges.is_empty(), "Graph should contain edges");

    // Additional checks for specific nodes and connections

    // Find the start event node
    let start_event = graph
        .nodes
        .iter()
        .find(|n| matches!(n.event, Some(BpmnEvent::Start(_))))
        .expect("There should be a Start Event");

    // Use assert_eq! with discriminant to check the event type
    assert_eq!(
        std::mem::discriminant(start_event.event.as_ref().unwrap()),
        std::mem::discriminant(&BpmnEvent::Start(String::new())),
        "Expected a Start Event"
    );

    // Similarly for the end event
    let end_event = graph
        .nodes
        .iter()
        .find(|n| matches!(n.event, Some(BpmnEvent::End(_))))
        .expect("There should be an End Event");

    assert_eq!(
        std::mem::discriminant(end_event.event.as_ref().unwrap()),
        std::mem::discriminant(&BpmnEvent::End(String::new())),
        "Expected an End Event"
    );

    // Checks for other types of events
    let gateway_event = graph
        .nodes
        .iter()
        .find(|n| matches!(n.event, Some(BpmnEvent::GatewayExclusive)))
        .expect("There should be an Exclusive Gateway");

    assert_eq!(
        std::mem::discriminant(gateway_event.event.as_ref().unwrap()),
        std::mem::discriminant(&BpmnEvent::GatewayExclusive),
        "Expected an Exclusive Gateway"
    );

    let tasks = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.event, Some(BpmnEvent::ActivityTask(_))))
        .collect::<Vec<_>>();
    assert!(
        !tasks.is_empty(),
        "There should be Activity Tasks in the graph"
    );
}

#[test]
fn test_uc2_generate_bpmn_diagram() {
    // UC2: Generate BPMN diagram based on input

    // Example DSL input
    let input = r#"
# Start Process
- Task 1
- Task 2
X
->BranchA
BranchA:
# Task A1
J JoinGateway
X->BranchB
BranchB:
# Task B1
J JoinGateway
X<-JoinGateway
# Task 3
. End Process
"#;

    // Initialize the lexer and parser
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    // Parse the input
    let parse_result = parser.parse();
    assert!(
        parse_result.is_ok(),
        "Parser should successfully process the UC2 input"
    );
    let mut graph = parse_result.unwrap();

    // Perform layout and generate BPMN
    // perform_layout(&mut graph);
    let bpmn_xml = generate_bpmn(&graph);

    // Verify that BPMN XML was generated
    assert!(
        !bpmn_xml.is_empty(),
        "BPMN XML should be generated based on the input"
    );
}
