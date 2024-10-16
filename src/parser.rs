use std::collections::HashMap;
use crate::lexer::{Token, Lexer};
use crate::ast::{BpmnEvent, BpmnGraph};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Parser {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token().unwrap_or(Token::Eof);
    }

    pub fn parse(&mut self) -> Result<BpmnGraph, String> {
        let mut graph = BpmnGraph::new();
        let mut last_node_id = None;
        let mut branches: HashMap<String, (usize, String)> = HashMap::new(); // branch label -> (node_id, branch text)
        let mut last_in_branch: HashMap<String, (usize, String)> = HashMap::new(); // Track the last node in each branch
        let mut join_gateway: HashMap<String, Vec<(usize, String)>> = HashMap::new(); // join label -> list of node_ids to join
        let mut current_branch: Option<String> = None;
        let mut gateway_stack: Vec<Vec<String>> = Vec::new(); // Stack for managing nested branches

        while self.current_token != Token::Eof {
            match &self.current_token {
                Token::EventStart(label) => {
                    let node_id = graph.add_node(BpmnEvent::Start(label.clone()));
                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id, None);
                    }
                    last_node_id = Some(node_id);
                }

                Token::EventMiddle(label) => {
                    let node_id = graph.add_node(BpmnEvent::Middle(label.clone()));
                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id, None);
                    }
                    last_node_id = Some(node_id);
                }

                Token::EventEnd(label) => {
                    let node_id = graph.add_node(BpmnEvent::End(label.clone()));
                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id, None);
                    }
                    last_node_id = Some(node_id);
                }

                Token::GatewayExclusive => {
                    let node_id = graph.add_node(BpmnEvent::GatewayExclusive);
                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id, None);
                    }
                    last_node_id = Some(node_id);
                    gateway_stack.push(Vec::new()); // Start a new set of branches
                }

                Token::Branch(label, text) => {
                    if let Some(prev_id) = last_node_id {
                        branches.insert(label.clone(), (prev_id, text.clone()));
                    } else {
                        return Err(format!("No previous node found for branch '{}'", label));
                    }
                    // Track the branch for the current gateway level
                    if let Some(gateway_branches) = gateway_stack.last_mut() {
                        gateway_branches.push(label.clone());
                    }
                }

                Token::Label(label) => {
                    // Switch to processing the branch with the given label
                    current_branch = Some(label.clone());
                    // Initialize the branch with the correct starting point
                    if let Some((branch_start_node, _branch_text)) = branches.get(label) {
                        last_in_branch.insert(label.clone(), (*branch_start_node, _branch_text.to_string()));
                    }
                }

                Token::ActivityTask(label) => {
                    let node_id = graph.add_node(BpmnEvent::ActivityTask(label.clone()));
                    if let Some(branch_label) = &current_branch {
                        if let Some(last_branch_node) = last_in_branch.get(branch_label) {
                            let edge_text = if last_branch_node.1.is_empty() { None } else { Some(last_branch_node.1.clone()) };
                            graph.add_edge(last_branch_node.0, node_id, edge_text); // Connect the last node in branch
                            last_in_branch.insert(branch_label.clone(), (node_id, String::new())); // Update last node in branch
                        } else {
                            return Err(format!("No start node found for branch '{}'", branch_label));
                        }
                    } else if let Some(prev_id) = last_node_id {
                        graph.add_edge(prev_id, node_id, None);
                    }
                    last_node_id = Some(node_id);
                }

                Token::Join(label, text) => {
                    // Record the last node in the current branch as a node to join later
                    if let Some(branch_label) = &current_branch {
                        if let Some(last_branch_node) = last_in_branch.get(branch_label) {
                            join_gateway
                                .entry(label.clone())
                                .or_insert_with(Vec::new)
                                .push((last_branch_node.0, text.clone()));
                        } else {
                            return Err(format!("No last node found in branch '{}' for join '{}'", branch_label, label));
                        }
                    }
                }

                Token::GatewayJoin(label) => {
                    let node_id = graph.add_node(BpmnEvent::GatewayJoin(label.clone()));
                    if let Some(joined_nodes) = join_gateway.remove(label) {
                        for prev_id in joined_nodes {
                            let edge_text = if prev_id.1.is_empty() { None } else { Some(prev_id.1.clone()) };
                            graph.add_edge(prev_id.0, node_id, edge_text);
                        }
                    } else {
                        return Err(format!("No join recorded for label '{}'", label));
                    }
                    last_node_id = Some(node_id);
                    
                    // Check if we've finished processing all branches for this gateway
                    if let Some(gateway_branches) = gateway_stack.pop() {
                        if gateway_branches.is_empty() {
                            return Err(format!("No branches found for gateway '{}'", label));
                        }
                    }
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