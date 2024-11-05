// src/main.rs

mod parser;
mod lexer;  
mod common;
mod to_xml;
mod layout;
// mod read_input;

use layout::{assign_bend_points::assign_bend_points, crossing_minimization::reduce_crossings, node_positioning::assign_xy_to_nodes, solve_layer_assignment::solve_layer_assignment};
// use common::graph;
// use layout::solve_layer_assignment;
use to_xml::generate_bpmn;
use lexer::Lexer;
use parser::Parser;
use common::bpmn_event::BpmnEvent;


fn main() {
//     let input = r#"
// = Pool 1
// == Lane 1
// # Start Event 
// G ->jump
// - Random Task
// == Lane 2
// # Start Event2
// G <-jump
// - Activity Task
//     "#;



    let input = r#"
# Start Event
X ->above"Go Here" ->below"No, here!" 

above: 
- Above
X ->above2"Go Here" ->below2"No, here!" 

above2:
- Above2
J endjoin2

below2:
- Below2
J endjoin2

<-endjoin2
J endjoin 

below: 
- And beyond
J endjoin 

<-endjoin
. Finish
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
        Ok(mut graph) => {
            println!("Parsed BPMN Graph:");
            let layers = solve_layer_assignment(&graph);
            let new_layers = reduce_crossings(&mut graph, &layers);
            assign_xy_to_nodes(&mut graph, &new_layers);
            assign_bend_points(&mut graph);


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

            let bpmn = generate_bpmn(&graph);
        },
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
        },

    }
}
