// src/main.rs

mod parser;
mod lexer;  
mod to_xml;
mod common;
mod layout;
mod read_input;

use lexer::Lexer;
use parser::Parser;
use common::bpmn_event::BpmnEvent;
use crate::to_xml::generate_bpmn;
use layout::crossing_minimization::reduce_crossings;
use layout::node_positioning::assign_xy_to_nodes;
use layout::assign_bend_points::assign_bend_points;
use layout::solve_layer_assignment::solve_layer_assignment;
use crate::read_input::read_lines;

use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if the required argument is passed
    if args.len() < 2 {
        eprintln!("Usage: {} <input_string>", args[0]);
        std::process::exit(1);
    }

    let input_data = &args[1];

    let input = read_lines(input_data).unwrap();

    run_parser(&input);
//     layout::testlayout::run_test_layout();
}

pub fn run_parser(input: &str) {
    // let input = r#"
    // - Start Event
    // - Middle Event
    // - Activity Task
    // - Gateway Exclusive
    // - Activity Task
    // . End Event
    // "#;

    // Initialize the lexer with the input
    let lexer = Lexer::new(input);

    // Initialize the parser with the lexer
    let mut parser = Parser::new(lexer);

    // Parse the input and handle the result
    match parser.parse() {
        Ok(mut graph) => {
            println!("Parsed BPMN Graph:");
            
            let layers = solve_layer_assignment(&graph);

            println!("\nLayer assignment after back-edge elimination:");
            for (node_id, layer) in &layers {
                println!("Node {}: Layer {}", node_id, layer);
            }

            // Ristumiste minimeerimine
            let new_layers = reduce_crossings(&mut graph, &layers);

            // X-Y määramine
            assign_xy_to_nodes(&mut graph, &new_layers);

            println!("\nNode positions after X-Y assignment:");
            for node in &graph.nodes {
                println!("Node {}: x = {:?}, y = {:?}", node.id, node.x, node.y);
            }

            // Servade painutamine
            assign_bend_points(&mut graph);

            

            for node in &graph.nodes {
                if let Some(event) = &node.event {
                    match event {
                        BpmnEvent::Start(label) => println!("  Start Event: {} (ID: {})", label, node.id),
                        BpmnEvent::Middle(label) => println!("  Middle Event: {} (ID: {})", label, node.id),
                        BpmnEvent::End(label) => println!("  End Event: {} (ID: {})", label, node.id),
                        BpmnEvent::GatewayExclusive => println!("  GatewayExclusive Event (ID: {})", node.id),
                        BpmnEvent::ActivityTask(label) => println!("  ActivityTask: {} (ID: {})", label, node.id),
                        BpmnEvent::GatewayJoin(label) => println!("  GatewayJoin Event: {} (ID: {})", label, node.id),
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
            generate_bpmn(&graph);
        },
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
        },

    }
}
