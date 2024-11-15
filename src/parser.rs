use std::collections::HashMap;
use crate::lexer::{Token, Lexer};
use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::Graph;
use crate::common::edge::Edge;

struct ParseContext {
    last_node_id: Option<usize>,
    current_pool: Option<String>,
    current_lane: Option<String>,
}

struct ParseBranching {
    label_map: HashMap<String, Vec<(BpmnEvent,usize, Option<String>, Option<String>)>>, // Remember the events for each label <label name, (event, node id, pool, lane)>
    label_end_map: HashMap<String, (String, Option<String>)>, // Remember the join label for each branch label <label name, (join label name, optional text)>
    gateway_map: HashMap<usize, Vec<(String, Option<String>)>>, // Remember the branches for each gateway <node id, (label, optional text)>
    gateway_end_map: HashMap<usize, Vec<String>>, // Remember the join labels for each gateway <node id, <join label names>>
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Parser {
            lexer,
            current_token,
        }
    }

    /// Advances to the next token
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token().unwrap_or(Token::Eof);
    }

    /// Parses the input and returns a graph
    pub fn parse(&mut self) -> Result<Graph, String> {
        let mut graph = Graph::new(vec![], vec![]);

        let mut context = ParseContext {
            last_node_id: None,
            current_pool: None,
            current_lane: None,
        };

        let mut branching = ParseBranching {
            label_map: HashMap::new(), // (label, events)
            label_end_map: HashMap::new(), // (label, (join label, optional text))
            gateway_map: HashMap::new(), // (node id, labels)
            gateway_end_map: HashMap::new(), // (node id, <join labels>)
    
        };

        let mut go_from_map: HashMap<usize, Vec<(String, Option<String>)>> = HashMap::new(); // (node id, <(labels, optional texts)>)
        let mut go_to_map: HashMap<String, Vec<usize>> = HashMap::new(); // (label, node ids)

        // Parse the input
        while self.current_token != Token::Eof {
            // Match the current token and parse accordingly
            match &self.current_token {
                Token::Pool(label) => self.parse_pool(&mut context, label),
                Token::Lane(label) => self.parse_lane(&mut context, label),
                Token::Go => { self.parse_go(&mut graph, &mut context, &mut go_from_map, &mut go_to_map)?; continue; },
                Token::EventStart(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::Start(label.clone()))?,
                Token::EventMiddle(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::Middle(label.clone()))?,
                Token::EventEnd(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::End(label.clone()))?,
                Token::ActivityTask(label) => self.parse_task(&mut graph, &mut context, BpmnEvent::ActivityTask(label.clone()))?,
                Token::GatewayExclusive => { self.parse_gateway(&mut graph, &mut context, BpmnEvent::GatewayExclusive, &mut branching)?; continue; },
                Token::Label(label) => self.parse_label(&mut graph, &mut context, &mut branching, &label.clone(), &mut go_from_map, &mut go_to_map)?,
                _ => return Err(format!("Unexpected token: {:?}", self.current_token)),
            }
            self.advance();
        }

        for (gateway_from_id, labels) in branching.gateway_map {
            for (label, text) in labels {
                let events = branching.label_map.get(&label).expect("Label not found in label_map");
                let first_event = events.get(0).expect("No events found for label");
                let node_id = graph.add_node(first_event.0.clone(), first_event.1.clone(), first_event.2.clone(), first_event.3.clone());
                let edge = Edge::new(gateway_from_id, node_id, text.clone(), None, None);
                graph.add_edge(edge);
                context.last_node_id = Some(node_id);
                for event in &events[1..] {
                    let node_id = graph.add_node(event.0.clone(), event.1.clone(), event.2.clone(), event.3.clone());
                    if let BpmnEvent::GatewayExclusive = event.0 {
                        // Check if this gateway is in `gateway_end_map` (indicating a join gateway)
                        if branching.gateway_end_map.contains_key(&node_id) {
                            // Skip adding an edge to join gateways
                            context.last_node_id = Some(node_id);
                            continue;
                        }
                    }
                    let edge = Edge::new(context.last_node_id.unwrap(), node_id, None, None, None);
                    graph.add_edge(edge);
                    context.last_node_id = Some(node_id);
                }
                let end_label = branching.label_end_map.get(&label);
                let end_join_ids: Vec<usize> = branching.gateway_end_map.iter()
                    .filter_map(|(key, labels)| {
                        if labels.contains(&end_label.unwrap().0) {
                            Some(*key)
                        } else {
                            None
                        }
                    }).collect();
                for end_join_id in end_join_ids {
                    let edge = Edge::new(context.last_node_id.unwrap(), end_join_id, end_label.unwrap().1.clone(),None, None);
                    graph.add_edge(edge);
                }
            }
        }

        for (from_node_id, labels) in go_from_map {
            for (label, text) in labels {
                if let Some(to_node_ids) = go_to_map.get(&label) {
                    for to_node_id in to_node_ids {
                        let edge = Edge::new(from_node_id, *to_node_id, text.clone(), None, None);
                        graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Set the current pool
    fn parse_pool(&self, context: &mut ParseContext, label: &str) {
        context.current_pool = Some(label.to_string());
        context.current_lane = None;
        context.last_node_id = None;
    }

    /// Set the current lane
    fn parse_lane(&self, context: &mut ParseContext, label: &str) {
        context.current_lane = Some(label.to_string());
        context.last_node_id = None;
    }

    /// Parse a gateway
    fn parse_gateway(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent, branching: &mut ParseBranching) -> Result<(), String> {
        // Assign a unique node ID to this gateway
        let node_id = graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());
        self.advance();
        // Check if this gateway is a branching gateway or a join gateway
        if let Token::Branch(_, _) = self.current_token {
            // If there is no last node ID, this is the first node in the graph
            if context.last_node_id != None {
                let edge = Edge::new(context.last_node_id.unwrap(), node_id, None, context.current_pool.clone(), context.current_lane.clone());
                graph.add_edge(edge);
            }
            // Store the node_id and corresponding branches to a map
            context.last_node_id = Some(node_id);
            while let Token::Branch(label, text) = &self.current_token {
                branching.gateway_map.entry(node_id).or_insert(vec![]).push((label.clone(), if text.is_empty() { None } else { Some(text.clone()) }));
                self.advance();
            }
        } else if let Token::JoinLabel(_) = self.current_token {
            // Store the node_id and corresponding join labels to a map
            context.last_node_id = Some(node_id);
            while let Token::JoinLabel(label) = &self.current_token {
                branching.gateway_end_map.entry(node_id).or_insert(vec![]).push(label.clone());
                self.advance();
            }
        } else {
            return Err(format!("Expected an '->' or '<-' after 'X' token! Current token: {:?}", self.current_token))
        }
        Ok(())
    }

    /// Parse a branch label
    fn parse_label(&mut self, graph: &mut Graph, context: &mut ParseContext, branching: &mut ParseBranching, label: &str, go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, go_to_map: &mut HashMap<String, Vec<usize>>) -> Result<(), String>  {
        self.advance();
        // Save all events for this label
        let mut events: Vec<(BpmnEvent,usize, Option<String>, Option<String>)> = vec![]; // (event, node_id, pool, lane)
        while !matches!(self.current_token, Token::Join(_,_)) {
            let current_token = self.current_token.clone();
            match &current_token {
                // If the current token is a label, parse it recursively
                Token::Label(inner_label) => self.parse_label(graph, context, branching, &inner_label.clone(), go_from_map, go_to_map)?,
                Token::EventStart(label) => events.push((BpmnEvent::Start(label.clone()),graph.next_node_id(), context.current_pool.clone(), context.current_lane.clone())),
                Token::EventMiddle(label) => events.push((BpmnEvent::Middle(label.clone()),graph.next_node_id(), context.current_pool.clone(), context.current_lane.clone())),
                Token::EventEnd(label) => events.push((BpmnEvent::End(label.clone()),graph.next_node_id(), context.current_pool.clone(), context.current_lane.clone())),
                Token::ActivityTask(label) => events.push((BpmnEvent::ActivityTask(label.clone()),graph.next_node_id(), context.current_pool.clone(), context.current_lane.clone())),
                Token::Go => { self.parse_go(graph, context, go_from_map, go_to_map)?; continue; },
                Token::GatewayExclusive => {
                    // Assign a unique node ID to this gateway
                    let gateway_id = graph.next_node_id();
                    context.last_node_id = Some(gateway_id);

                    events.push((BpmnEvent::GatewayExclusive, gateway_id, context.current_pool.clone(), context.current_lane.clone()));

                    // Store the gateway_id and parse branches without advancing
                    self.advance();
                    let deferred_parse_gateway = self.deferred_parse_gateway(branching, gateway_id);
                    if deferred_parse_gateway.is_err() {
                        return deferred_parse_gateway;
                    }
                    continue;
                },
                _ => return Err(format!("Unexpected token in label ({:?}): {:?}", label,self.current_token)),
            }
            self.advance();
        }
        branching.label_map.insert(label.to_string(), events);
        if let Token::Join(exit_label, text) = &self.current_token {
            branching.label_end_map.insert(label.to_string(), (exit_label.clone(), if text.is_empty() { None } else { Some(text.clone()) }));
        } else {
            return Err(format!("Expected a join label after branch label! Current token: {:?}", self.current_token));
        }
        Ok(())
    }

    /// Parse a gateway inside a branch
    fn deferred_parse_gateway(&mut self, branching: &mut ParseBranching, gateway_id: usize) -> Result<(), String> {
        // Check if this gateway is a branching gateway or a join gateway
        if let Token::Branch(_, _) = self.current_token {
            // Store the node_id and corresponding branches to a map
            while let Token::Branch(branch_label, text) = &self.current_token {
                branching.gateway_map.entry(gateway_id)
                    .or_insert(vec![])
                    .push((branch_label.clone(), if text.is_empty() { None } else { Some(text.clone()) }));
                self.advance();
            }
        } else if let Token::JoinLabel(_) = self.current_token {
            // Store the node_id and corresponding join labels to a map
            while let Token::JoinLabel(join_label) = &self.current_token {
                branching.gateway_end_map.entry(gateway_id)
                    .or_insert(vec![])
                    .push(join_label.clone());
                self.advance();
            }
        } else {
            return Err(format!("Expected an '->' or '<-' after 'X' token! Current token: {:?}", self.current_token))
        }
        Ok(())
    }

    /// Common function to parse an event or task
    fn parse_common(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent) -> Result<(), String> {
        let node_id = graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());
        if let Some(last_node_id) = context.last_node_id {
            let edge = Edge::new(last_node_id, node_id, None, context.current_pool.clone(), context.current_lane.clone());
            graph.add_edge(edge);
        }
        context.last_node_id = Some(node_id);
        Ok(())
    }

    /// Parse an event
    fn parse_event(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent) -> Result<(), String> {
        self.parse_common(graph, context, event)
    }

    /// Parse a task
    fn parse_task(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent) -> Result<(), String> {
        self.parse_common(graph, context, event)
    }

    /// Parse a go
    fn parse_go(&mut self, graph: &mut Graph, context: &mut ParseContext, go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, go_to_map: &mut HashMap<String, Vec<usize>>) -> Result<(), String> {
        self.advance();
        // Check if this go is a branching go or a join go
        if let Token::Branch(_, _) = self.current_token {
            // Loop through all branches and store the labels and texts
            while let Token::Branch(label, text) = &self.current_token {
                if let Some(last_node_id) = context.last_node_id {
                    go_from_map.entry(last_node_id).or_insert(vec![]).push((label.clone(), if text.is_empty() { None } else { Some(text.clone()) }));
                } else {
                    return Err(format!("Expected a node before branch label! Current token: {:?}", self.current_token));
                }
                self.advance();
            }
        } else if let Token::JoinLabel(_) = self.current_token {
            // Loop through all join labels and store the node IDs
            let next_node_id = graph.last_node_id + 1;
            while let Token::JoinLabel(label) = &self.current_token {
                go_to_map.entry(label.clone()).or_insert(vec![]).push(next_node_id);
                self.advance();
            }
        } else {
            return Err(format!("Expected an '->' or '<-' after 'G' token! Current token: {:?}", self.current_token))
        }
        Ok(())
    }
}