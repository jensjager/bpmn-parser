use std::collections::HashMap;
use crate::lexer::{Token, Lexer};
use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::Graph;
use crate::common::edge::Edge;


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

    pub fn parse(&mut self) -> Result<Graph, String> {
        let mut graph = Graph::new(vec![], vec![]);
        let mut last_node_id = None;
        let mut branches: HashMap<String, (usize, String)> = HashMap::new(); // branch label -> (node_id, branch text)
        let mut go_from: HashMap<String, (usize, String)> = HashMap::new();
        let mut go_to: HashMap<String, usize> = HashMap::new();
        let mut last_in_branch: HashMap<String, (usize, String)> = HashMap::new(); // Track the last node in each branch
        let mut join_gateway: HashMap<String, Vec<(usize, String)>> = HashMap::new(); // join label -> list of node_ids to join
        let mut current_branch: Option<String> = None;
        let mut gateway_stack: Vec<Vec<String>> = Vec::new(); // Stack for managing nested branches
        let mut current_pool: Option<String> = None;
        let mut current_lane: Option<String> = None;

        while self.current_token != Token::Eof {
            match &self.current_token {
                Token::Pool(label) => {
                    current_pool = Some(label.clone());
                    current_lane = None;
                    last_node_id = None;
                }
    
                Token::Lane(label) => {
                    current_lane = Some(label.clone());
                    last_node_id = None;
                }
                
                Token::EventStart(label) => {
                    let node_id = graph.add_node_noid(BpmnEvent::Start(label.clone()), current_pool.clone(), current_lane.clone());

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(Edge::new(prev_id, node_id, None, current_pool.clone(), current_lane.clone()));
                    }
                    last_node_id = Some(node_id);
                }

                Token::EventMiddle(label) => {
                    let node_id = graph.add_node_noid(BpmnEvent::Middle(label.clone()), current_pool.clone(), current_lane.clone());

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(Edge::new(prev_id, node_id, None, current_pool.clone(), current_lane.clone()));
                    }
                    last_node_id = Some(node_id);
                }

                Token::EventEnd(label) => {
                    let node_id = graph.add_node_noid(BpmnEvent::End(label.clone()), current_pool.clone(), current_lane.clone());

                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(Edge::new(prev_id, node_id, None, current_pool.clone(), current_lane.clone()));
                    }
                    last_node_id = Some(node_id);
                }

                Token::GatewayExclusive => {
                    let node_id: usize = graph.add_node_noid(BpmnEvent::GatewayExclusive, current_pool.clone(), current_lane.clone());
                    if let Some(prev_id) = last_node_id {
                        graph.add_edge(Edge::new(prev_id, node_id, None, current_pool.clone(), current_lane.clone()));
                    }
                    last_node_id = Some(node_id);
                    gateway_stack.push(Vec::new()); // Start a new set of branches
                }

                Token::GoFrom(label, text) => {
                    go_from.insert(label.clone(), (last_node_id.unwrap(), text.clone()));
                }

                Token::GoTo(label) => {
                    go_to.insert(label.clone(), last_node_id.unwrap());
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
                    let node_id = graph.add_node_noid(BpmnEvent::ActivityTask(label.clone()), current_pool.clone(), current_lane.clone());
                    
                    if let Some(branch_label) = &current_branch {
                        if let Some(last_branch_node) = last_in_branch.get(branch_label) {
                            let edge_text = if last_branch_node.1.is_empty() { None } else { Some(last_branch_node.1.clone()) };
                            graph.add_edge(Edge::new(last_branch_node.0, node_id, edge_text, current_pool.clone(), current_lane.clone())); // Connect the last node in branch
                            last_in_branch.insert(branch_label.clone(), (node_id, String::new())); // Update last node in branch
                        } else {
                            return Err(format!("No start node found for branch '{}'", branch_label));
                        }
                    } else if let Some(prev_id) = last_node_id {
                        graph.add_edge(Edge::new(prev_id, node_id, None, current_pool.clone(), current_lane.clone()));
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
                    let node_id = graph.add_node_noid(BpmnEvent::GatewayJoin(label.clone()), current_pool.clone(), current_lane.clone());
                    if let Some(joined_nodes) = join_gateway.remove(label) {
                        for prev_id in joined_nodes {
                            let edge_text = if prev_id.1.is_empty() { None } else { Some(prev_id.1.clone()) };
                            graph.add_edge(Edge::new(prev_id.0, node_id, edge_text, current_pool.clone(), current_lane.clone()));
                        }
                    } else {
                        return Err(format!("No join recorded for label '{}'", label));
                    }
                    last_node_id = Some(node_id);
                    current_branch = None;
                    
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

        for (label, (from_id, text)) in go_from {
            if let Some(to_id) = go_to.get(&label) {
                let edge_text = if text.is_empty() { None } else { Some(text.clone()) };
                graph.add_edge(Edge::new(from_id, *to_id, edge_text, current_pool.clone(), current_lane.clone()));
            } else {
                return Err(format!("No 'go to' found for label '{}'", label));
            }
        }

        Ok(graph)
    }
}