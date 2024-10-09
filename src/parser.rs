// src/parser.rs

use crate::lexer::{Token, Lexer};
use crate::ast::{BpmnNode, BpmnEvent, BpmnFlow};

// Struct for the Parser
pub struct Parser<'a> {
    lexer: Lexer<'a>,            // The lexer that provides tokens
    current_token: Option<Token>,// Holds the current token
    has_seen_start_event: bool,  // Flag to track if start event has been processed
}

impl<'a> Parser<'a> {
    // Create a new parser instance
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
            has_seen_start_event: false, // Start with no events seen
        }
    }

    // Advance to the next token
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    // Parse the entire BPMD file into an AST
    pub fn parse(&mut self) -> Result<BpmnNode, String> {
        let mut nodes = Vec::new();

        while let Some(token) = self.current_token.clone() { // Clone to avoid borrowing
            match token {
                Token::EventStart => {
                    if self.has_seen_start_event {
                        // If we've seen the start event, treat this as a middle event
                        let event = self.parse_middle_event()?;
                        nodes.push(event);
                    } else {
                        // This is the first event, so treat it as the start event
                        let event = self.parse_start_event()?;
                        nodes.push(event);
                        self.has_seen_start_event = true; // Mark that start event has been processed
                    }
                },
                Token::EventEnd => {
                    let event = self.parse_end_event()?;
                    nodes.push(event);
                },
                Token::Eof => break,
                _ => return Err(format!("Unexpected token: {:?}", token)),
            }
            self.advance();
        }

        Ok(BpmnNode::Flow(BpmnFlow::Nodes(nodes))) // Return a Flow node containing all parsed nodes
    }

    // Parse a start event (e.g. `- Start Event`)
    fn parse_start_event(&mut self) -> Result<BpmnNode, String> {
        // Ensure the current token is a start event
        if let Some(Token::EventStart) = self.current_token {
            // Advance to the next token
            self.advance();
            if let Some(Token::Text(text)) = self.current_token.clone() {
                Ok(BpmnNode::Event(BpmnEvent::Start(text)))
            } else {
                Err("Expected text after start event".to_string())
            }
        } else {
            Err("Expected a start event".to_string())
        }
    }

    // Parse a middle event (e.g. `- Middle Event`)
    fn parse_middle_event(&mut self) -> Result<BpmnNode, String> {
        if let Some(Token::EventStart) = self.current_token {
            self.advance();
            if let Some(Token::Text(text)) = self.current_token.clone() {
                Ok(BpmnNode::Event(BpmnEvent::Middle(text)))
            } else {
                Err("Expected text after middle event".to_string())
            }
        } else {
            Err("Expected a middle event".to_string())
        }
    }

    // Parse an end event (e.g. `. End Event`)
    fn parse_end_event(&mut self) -> Result<BpmnNode, String> {
        if let Some(Token::EventEnd) = self.current_token {
            // Advance to the next token
            self.advance();
            if let Some(Token::Text(text)) = self.current_token.clone() {
                Ok(BpmnNode::Event(BpmnEvent::End(text)))
            } else {
                Err("Expected text after end event".to_string())
            }
        } else {
            Err("Expected an end event".to_string())
        }
    }
}
