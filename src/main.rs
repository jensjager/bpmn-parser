// src/main.rs

mod parser;
mod ast;
mod lexer;  

use lexer::Lexer;
use parser::Parser;
use ast::{BpmnNode, BpmnEvent, BpmnFlow};

fn main() {
    let input = r#"
    - Start Event
    - Middle Event
    . End Event
    "#;

    // Initialize the lexer with the input
    let lexer = Lexer::new(input);

    // Initialize the parser with the lexer
    let mut parser = Parser::new(lexer);

    // Parse the input and handle the result
    match parser.parse() {
        Ok(ast) => {
            println!("Parsed AST:");
            print_ast(&ast, 0);
        },
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
        },
    }
}

// Helper function to print the AST in a readable format
fn print_ast(node: &BpmnNode, indent: usize) {
    let indentation = "  ".repeat(indent);
    match node {
        BpmnNode::Event(event) => {
            match event {
                BpmnEvent::Start(label) => println!("{}Start Event: {}", indentation, label),
                BpmnEvent::Middle(label) => println!("{}Middle Event: {}", indentation, label),
                BpmnEvent::End(label) => println!("{}End Event: {}", indentation, label),
            }
        },
        BpmnNode::Flow(flow) => {
            match flow {
                BpmnFlow::Nodes(nodes) => {
                    println!("{}Flow:", indentation);
                    for n in nodes {
                        print_ast(n, indent + 1);
                    }
                },
            }
        },
    }
}
