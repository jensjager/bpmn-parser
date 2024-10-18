// src/main.rs

mod parser;
mod ast;
mod lexer;  

use common::graph;
use lexer::Lexer;
use parser::Parser;
use ast::Ast;
use common::bpmn_event::BpmnEvent;

mod common;
mod layout;

fn main() {
    // run_parser();
    layout::testlayout::run_test_layout();
}

pub fn run_parser() {
    let input = r#"
    - Start Event
    - Middle Event
    . End Event
    "#;

    // Initialize the lexer with the input
    let lexer = Lexer::new(input);

    // Initialize the parser with the lexer
    let mut parser = Parser::new(lexer);

    let ast = Ast::new();
    // Parse the input and handle the result
    match parser.parse() {
        Ok(mut graph) => {
            println!("Parsed BPMN Graph:");
            
            println!("Before back-edge elimination:");
            for edge in &graph.edges {
                println!("Edge from {} to {}", edge.from, edge.to);
            }

            graph.eliminate_back_edges();

            println!("After back-edge elimination:");
            for edge in &graph.edges {
                println!("Edge from {} to {}", edge.from, edge.to);
            }

            println!("\nPrinting graph:");

            

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
        },
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
        },
    }
}
