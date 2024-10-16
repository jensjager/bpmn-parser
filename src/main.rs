// src/main.rs

mod parser;
mod ast;
mod lexer;  

use lexer::Lexer;
use parser::Parser;

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
        Ok(graph) => {
            println!("Parsed BPMN Graph:");
            graph.print_graph();
        },
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
        },
    }
}

