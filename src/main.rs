// src/main.rs

mod parser;
mod lexer;  
mod common;
// mod layout;
// mod read_input;

// use common::graph;
// use layout::solve_layer_assignment;
use lexer::Lexer;
use parser::Parser;
use common::bpmn_event::BpmnEvent;


fn main() {
    let input = r#"
  = Pool 1
  == Lane 1
  - some node
  G ->label
  label2:
  - random task
  == Lane 2
  # Start Event
  label:
  # Middle Event
  - Activity Task
  G ->label2
  X ->above"Go Here" ->below"No, here!"
  above: 
  - Above
  J endjoin 
  
  below: 
  - And beyond
  J endjoin 

  <-endjoin
  - Activity Task

  = Pool 2
  - some node2
  = Pool 1
  == Lane 2
  # End Event
    "#;

    run_parser(&input);
}

pub fn run_parser(input: &str) {

    // Initialize the lexer with the input
    let lexer = Lexer::new(input);

    // Initialize the parser with the lexer
    let mut parser = Parser::new(lexer);

    // Parse the input and handle the result
    match parser.parse() {
        Ok(graph) => {
            println!("Parsed BPMN Graph:");

            for node in &graph.nodes {
                if let Some(event) = &node.event {
                    match event {
                        BpmnEvent::Start(label) => println!("  Start Event: {} (ID: {}) Pool: {:?}; Lane: {:?}", label, node.id, node.pool.as_deref().unwrap_or("None"), node.lane.as_deref().unwrap_or("None")),
                        BpmnEvent::Middle(label) => println!("  Middle Event: {} (ID: {}) Pool: {:?}; Lane: {:?}", label, node.id, node.pool.as_deref().unwrap_or("None"), node.lane.as_deref().unwrap_or("None")),
                        BpmnEvent::End(label) => println!("  End Event: {} (ID: {}) Pool: {:?}; Lane: {:?}", label, node.id, node.pool.as_deref().unwrap_or("None"), node.lane.as_deref().unwrap_or("None")),
                        BpmnEvent::GatewayExclusive => println!("  GatewayExclusive Event (ID: {}) Pool: {:?}; Lane: {:?}", node.id, node.pool.as_deref().unwrap_or("None"), node.lane.as_deref().unwrap_or("None")),
                        BpmnEvent::ActivityTask(label) => println!("  ActivityTask: {} (ID: {}) Pool: {:?}; Lane: {:?}", label, node.id, node.pool.as_deref().unwrap_or("None"), node.lane.as_deref().unwrap_or("None")),
                        BpmnEvent::GatewayJoin(label) => println!("  GatewayJoin Event: {} (ID: {}) Pool: {:?}; Lane: {:?}", label, node.id, node.pool.as_deref().unwrap_or("None"), node.lane.as_deref().unwrap_or("None")),
                    }
                } else {
                    println!("  No Event (ID: {})", node.id);
                }
            }
    
            println!("Edges:");
            for edge in &graph.edges {
                if let Some(text) = &edge.text {
                    println!("  From Node {} to Node {}: '{}'", edge.from, edge.to, text);
                } else {
                    println!("  From Node {} to Node {}", edge.from, edge.to);
                }
            }
            // let bpmn = generate_bpmn(&graph);
        },
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
        },

    }
}
