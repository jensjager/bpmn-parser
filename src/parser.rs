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
                Token::EventStart => {
                    self.advance();
                    let label = self.parse_text()?;  // Get the label after the event
                    let node_id = graph.add_node(BpmnEvent::Start(label));

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id); // Connect previous node
                    }
                    last_node_id = Some(node_id);
                }
                Token::EventMiddle => {
                    self.advance();
                    let label = self.parse_text()?;  // Get the label after the event
                    let node_id = graph.add_node(BpmnEvent::Middle(label));

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id); // Connect previous node
                    }
                    last_node_id = Some(node_id);
                }
                Token::EventEnd => {
                    self.advance();
                    let label = self.parse_text()?;  // Get the label after the event
                    let node_id = graph.add_node(BpmnEvent::End(label));

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

    // Helper to parse the text following an event symbol
    fn parse_text(&mut self) -> Result<String, String> {
        if let Token::Text(label) = &self.current_token {
            let result = label.clone();
            Ok(result)
        } else {
            Err(format!("Expected text after event, found: {:?}", self.current_token))
        }
    }
}