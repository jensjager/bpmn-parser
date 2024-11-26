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
    graph: Graph,
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Parser {
            graph: Graph::new(vec![], vec![]),
            lexer,
            current_token,
        }
    }

    /// Advances to the next token
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token().unwrap_or(Token::Eof);
    }

    /// Peeks at the next token without advancing
    fn peek(&mut self) -> Token {
        self.lexer.peek_token()
    }

    /// Parses the input and returns a graph
    pub fn parse(&mut self) -> Result<Graph, String> {
        // Initialize the context and branching structures
        let mut context = ParseContext {
            last_node_id: None,
            current_pool: None,
            current_lane: None,
        };

        // Initialize the branching structure
        let mut branching = ParseBranching {
            label_map: HashMap::new(), // (label, events)
            label_end_map: HashMap::new(), // (label, (join label, optional text))
            gateway_map: HashMap::new(), // (node id, labels)
            gateway_end_map: HashMap::new(), // (node id, <join labels>)
    
        };

        // Initialize the go structures
        let mut go_from_map: HashMap<usize, Vec<(String, Option<String>)>> = HashMap::new(); // (node id, <(labels, optional texts)>)
        let mut go_to_map: HashMap<String, Vec<usize>> = HashMap::new(); // (label, node ids)
        let mut go_active = false; // Flag to indicate if a go is active (outgoing)

        // Parse the input
        while self.current_token != Token::Eof {
            if let Err(err) = self.check_go_active_error(go_active) {
                return Err(err);
            }

            // Match the current token and parse accordingly
            let current_token = self.current_token.clone();
            match current_token {
                Token::Pool(label) => self.parse_pool(&mut context, &label, &mut go_active),
                Token::Lane(label) => self.parse_lane(&mut context, &label, &mut go_active),
                Token::Go => { self.parse_go(context.last_node_id, &mut go_from_map, &mut go_to_map, &mut go_active)?; continue; },
                Token::EventStart(label) => self.parse_common( &mut context, BpmnEvent::Start(label.clone()))?,
                Token::EventMiddle(label) => self.parse_common(&mut context, BpmnEvent::Middle(label.clone()))?,
                Token::EventEnd(label) => self.parse_common(&mut context, BpmnEvent::End(label.clone()))?,
                Token::ActivityTask(label) => self.parse_common(&mut context, BpmnEvent::ActivityTask(label.clone()))?,
                Token::GatewayExclusive => { self.parse_gateway(&mut context, BpmnEvent::GatewayExclusive, &mut branching)?; continue; },
                Token::Label(label) => self.parse_label(&mut context, &mut branching, &label.clone(), &mut go_from_map, &mut go_to_map, &mut go_active)?,
                Token::Error(message) => return Err(message.clone()),
                _ => return Err(format!("Unexpected token: {:?}", self.current_token)),
            }
            self.advance();
        }

        for (gateway_from_id, labels) in branching.gateway_map {
            for (label, text) in labels {
                let events = branching.label_map.get(&label).expect("Label not found in label_map");
                let first_event = events.get(0).expect("No events found for label");
                let node_id = self.graph.add_node(first_event.0.clone(), first_event.1.clone(), first_event.2.clone(), first_event.3.clone());
                let edge = Edge::new(gateway_from_id, node_id, text.clone());
                self.graph.add_edge(edge);
                context.last_node_id = Some(node_id);
                for event in &events[1..] {
                    let node_id = self.graph.add_node(event.0.clone(), event.1.clone(), event.2.clone(), event.3.clone());
                    if let BpmnEvent::GatewayExclusive = event.0 {
                        // Check if this gateway is in `gateway_end_map` (indicating a join gateway)
                        if branching.gateway_end_map.contains_key(&node_id) {
                            // Skip adding an edge to join gateways
                            context.last_node_id = Some(node_id);
                            continue;
                        }
                    }
                    let edge = Edge::new(context.last_node_id.unwrap(), node_id, None);
                    self.graph.add_edge(edge);
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
                    let edge = Edge::new(context.last_node_id.unwrap(), end_join_id, end_label.unwrap().1.clone());
                    self.graph.add_edge(edge);
                }
            }
        }

        for (from_node_id, labels) in go_from_map {
            for (label, text) in labels {
                if let Some(to_node_ids) = go_to_map.get(&label) {
                    for to_node_id in to_node_ids {
                        let edge = Edge::new(from_node_id, *to_node_id, text.clone());
                        self.graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(self.graph.clone())
    }

    /// Set the current pool
    fn parse_pool(&mut self, context: &mut ParseContext, label: &str, go_active: &mut bool) {
        context.current_pool = Some(label.to_string());
        context.current_lane = None;
        context.last_node_id = None;
        self.lexer.seen_start = false;
        *go_active = false;
    }

    /// Set the current lane
    fn parse_lane(&mut self, context: &mut ParseContext, label: &str, go_active: &mut bool) {
        context.current_lane = Some(label.to_string());
        context.last_node_id = None;
        self.lexer.seen_start = false;
        *go_active = false;
    }

    /// Parse a gateway
    fn parse_gateway(
        &mut self, 
        context: &mut ParseContext, 
        event: BpmnEvent, 
        branching: &mut ParseBranching
    ) -> Result<(), String> {
        // Assign a unique node ID to this gateway
        let node_id = self.graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());

        // Call the common gateway parsing logic
        return self.parse_gateway_common(context, node_id, branching, false)
    }
    
    /// Common logic for parsing gateways (branch or join)
    fn parse_gateway_common(
        &mut self, 
        context: &mut ParseContext, 
        node_id: usize, 
        branching: &mut ParseBranching,
        inside_label: bool
    ) -> Result<(), String> {
        // Save the current line and error message in case of an error
        let line = self.lexer.line.clone();
        let error_message = self.lexer.highlight_error();

        self.advance();
        
        // Handle Branch or Join for the gateway
        match &self.current_token {
            Token::Branch(_, _) => self.handle_gateway_branching(context, node_id, branching, inside_label)?,
            Token::JoinLabel(_) => self.handle_gateway_join(context, node_id, branching, inside_label)?,
            Token::Error(ref message) => return Err(message.clone()),
            _ => return Err(format!(
                "Expected Branch or Join after 'X' token at line {:?} \n{}", line, error_message)),
        }

        Ok(())
    }

    // Helper to handle branching
    fn handle_gateway_branching(
        &mut self,
        context: &mut ParseContext,
        node_id: usize,
        branching: &mut ParseBranching,
        inside_label: bool
    ) -> Result<(), String> {
        if !inside_label {
            self.connect_nodes(context, node_id);
        }
        while let Token::Branch(label, text) = &self.current_token {
            let branch_text = if text.is_empty() { None } else { Some(text.clone()) };
            branching
                .gateway_map
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push((label.clone(), branch_text));
            self.advance();
        }
        Ok(())
    }

    // Helper to handle joins
    fn handle_gateway_join(
        &mut self,
        context: &mut ParseContext,
        node_id: usize,
        branching: &mut ParseBranching,
        inside_label: bool
    ) -> Result<(), String> {
        if !inside_label {
            context.last_node_id = Some(node_id);
        }
        while let Token::JoinLabel(label) = &self.current_token {
            branching
                .gateway_end_map
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push(label.clone());
            self.advance();
        }
        Ok(())
    }

    /// Parse a branch label
    fn parse_label(
        &mut self, 
        context: &mut ParseContext, 
        branching: &mut ParseBranching, label: &str, 
        go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, 
        go_to_map: &mut HashMap<String, Vec<usize>>, 
        go_active: &mut bool
    ) -> Result<(), String>  {
        self.advance();

        // Save all events for this label
        let mut events: Vec<(BpmnEvent,usize, Option<String>, Option<String>)> = vec![]; // (event, node_id, pool, lane)

        while !matches!(self.current_token, Token::Join(_,_)) {
            let current_token = self.current_token.clone();
            match &current_token {
                // If the current token is a label, parse it recursively
                Token::Label(inner_label) => {
                    self.parse_label(context, branching, &inner_label, go_from_map, go_to_map, go_active)?;
                }
                Token::EventStart(label) => {
                    events.push(self.create_event_node(context, BpmnEvent::Start(label.clone()))?);
                }
                Token::EventMiddle(label) => {
                    events.push(self.create_event_node(context, BpmnEvent::Middle(label.clone()))?);
                }
                Token::EventEnd(label) => {
                    events.push(self.create_event_node(context, BpmnEvent::End(label.clone()))?);
                }
                Token::ActivityTask(label) => {
                    events.push(self.create_event_node(context, BpmnEvent::ActivityTask(label.clone()))?);
                }
                Token::Go => { 
                    let from_id = events.last().map(|event| event.1);
                    self.parse_go(from_id, go_from_map, go_to_map, go_active)?; 
                    continue; 
                },
                Token::GatewayExclusive => {
                    self.handle_gateway_in_label(context, branching, &mut events)?;
                    continue;
                },
                Token::Error(message) => return Err(message.clone()),
                _ => return Err(format!("Unexpected token in label ({:?}): {:?}", label,self.current_token)),
            }
            self.advance();
        }
        branching.label_map.insert(label.to_string(), events);
        if let Token::Join(exit_label, text) = &self.current_token {
            branching.label_end_map.insert(
                label.to_string(),
                (
                    exit_label.clone(), 
                    if text.is_empty() { None } else { Some(text.clone()) }
                )
            );
        } else {
            return Err(format!(
                "Expected a join label after branch label! Current token: {:?}", 
                self.current_token
            ));
        }
        *go_active = false;
        Ok(())
    }

    /// Create an event node
    fn create_event_node(
        &mut self,
        context: &ParseContext,
        event: BpmnEvent,
    ) -> Result<(BpmnEvent, usize, Option<String>, Option<String>), String> {
        let node_id = self.graph.next_node_id();
        Ok((
            event,
            node_id,
            context.current_pool.clone(),
            context.current_lane.clone(),
        ))
    }    

    /// Handle a gateway inside a branch
    fn handle_gateway_in_label(
        &mut self,
        context: &mut ParseContext,
        branching: &mut ParseBranching,
        events: &mut Vec<(BpmnEvent, usize, Option<String>, Option<String>)>,
    ) -> Result<(), String> {
        // Assign a unique node ID to this gateway
        let gateway_id = self.graph.next_node_id();
        context.last_node_id = Some(gateway_id);
        
        events.push((
            BpmnEvent::GatewayExclusive,
            gateway_id,
            context.current_pool.clone(),
            context.current_lane.clone(),
        ));
    
        // Store the gateway_id and parse branches without advancing
        return self.parse_gateway_common(context, gateway_id, branching, true);
    }

    /// Connect two nodes with an edge if needed
    fn connect_nodes(&mut self, context: &mut ParseContext, node_id: usize) {
        if let Some(last_node_id) = context.last_node_id {
            let edge = Edge::new(last_node_id, node_id, None);
            self.graph.add_edge(edge);
        }
        context.last_node_id = Some(node_id);
    }

    /// Common function to parse an event or task
    fn parse_common(&mut self, context: &mut ParseContext, event: BpmnEvent) -> Result<(), String> {
        let node_id = self.graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());
        self.connect_nodes(context, node_id);
        Ok(())
    }

    /// Parse a go
    fn parse_go(
        &mut self, 
        from_id: Option<usize>,
        go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, 
        go_to_map: &mut HashMap<String, Vec<usize>>, 
        go_active: &mut bool
    ) -> Result<(), String> {
        // Save the current line and error message in case of an error
        let line = self.lexer.line.clone();
        let error_message = self.lexer.highlight_error();

        // Check if this go is a branching go or a join go
        self.advance();
        match &self.current_token {
            Token::Branch(_, _) => {
                *go_active = true;
                self.handle_go_from(from_id, go_from_map)?;
            }
            Token::JoinLabel(_) => {
                *go_active = false;
                let next_node_id = self.graph.last_node_id + 1;
                self.handle_go_to(next_node_id, go_to_map)?;
            }
            Token::Error(message) => return Err(message.clone()),
            _ => {
                return Err(format!(
                    "Expected a 'Branch' or 'JoinLabel' after 'G' token at line {} \n{}",
                    line, error_message
                ));
            }
        }
        Ok(())
    }

    fn handle_go_from(
        &mut self,
        from_id: Option<usize>,
        go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>,
    ) -> Result<(), String> {
        while let Token::Branch(label, text) = &self.current_token {
            let last_node_id = from_id.ok_or_else(|| {
                format!(
                    "Expected a node before 'G' token at line {}!\n{}",
                    self.lexer.line,
                    self.lexer.highlight_error()
                )
            })?;
    
            let edge_text = if text.is_empty() { None } else { Some(text.clone()) };
            go_from_map.entry(last_node_id).or_insert_with(Vec::new).push((label.clone(), edge_text));
    
            self.advance();
        }
    
        Ok(())
    }
    
    /// Handle parsing for a join 'Go' token
    fn handle_go_to(
        &mut self,
        next_node_id: usize,
        go_to_map: &mut HashMap<String, Vec<usize>>,
    ) -> Result<(), String> {    
        // Check that a valid node type follows
        let next_token = self.peek();
        if !self.is_token_a_node(next_token) {
            return Err(format!(
                "Cannot end with an outgoing 'G' token at line {} \n{}",
                self.lexer.line,
                self.lexer.highlight_error()
            ));
        }
    
        // Loop through all join labels and store the node IDs
        while let Token::JoinLabel(label) = &self.current_token {
            go_to_map.entry(
                label.clone())
                .or_insert_with(Vec::new)
                .push(next_node_id);
            self.advance();
        }
    
        Ok(())
    }

    fn is_token_a_node(&self, token: Token) -> bool {
        matches!(
            token,
            Token::EventStart(_)    | 
            Token::EventMiddle(_)   | 
            Token::EventEnd(_)      | 
            Token::ActivityTask(_)  | 
            Token::GatewayExclusive
        )
    }

    fn check_go_active_error(&mut self, go_active: bool) -> Result<(), String> {
        if go_active && self.is_token_a_node(self.current_token.clone()) {
            return Err(format!(
                "Expected a Join for 'G' token before defining a new node at line {:?}\n{}",
                self.lexer.line, self.lexer.highlight_error()
            ));
        }
        Ok(())
    }
}