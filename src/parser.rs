// src/parser.rs

use crate::lexer::{Token, Lexer};
use crate::ast::{BpmnEvent, BpmnGraph};

// Struct for the Parser
pub struct Parser<'a> {
    lexer: Lexer<'a>,            // The lexer that provides tokens
    current_token: Token,         // Current token being processed
}

impl<'a> Parser<'a> {
    // Create a new parser from the lexer
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Parser { lexer, current_token }
    }

    // Function to advance the current token
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token().unwrap_or(Token::Eof);
    }

    // Main parsing function
    pub fn parse(&mut self) -> Result<BpmnGraph, String> {
        let mut graph = BpmnGraph::new();
        let mut last_node_id = None;

        while self.current_token != Token::Eof {
            match &self.current_token {
                Token::EventStart(label) => {
                    let node_id = graph.add_node(BpmnEvent::Start(label.clone()));

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id); // Connect previous node
                    }
                    last_node_id = Some(node_id);
                }
                Token::EventMiddle(label) => {
                    let node_id = graph.add_node(BpmnEvent::Middle(label.clone()));

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id); // Connect previous node
                    }
                    last_node_id = Some(node_id);
                }
                Token::EventEnd(label) => {
                    let node_id = graph.add_node(BpmnEvent::End(label.clone()));

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id); // Connect previous node
                    }
                    last_node_id = Some(node_id);
                }
                _ => {
                    return Err(format!("Unexpected token: {:?}", self.current_token));
                }
            }

            self.advance();
        }

        Ok(graph)
    }
}