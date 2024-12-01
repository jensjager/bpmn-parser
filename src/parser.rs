use std::collections::HashMap;
use crate::lexer::{Lexer, LexerError, Token};
use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::Graph;
use crate::common::edge::Edge;

struct ParseContext {
    last_node_id: Option<usize>,
    current_pool: Option<String>,
    current_lane: Option<String>,
    current_token: Token,
}

struct ParseBranching {
    label_map: HashMap<String, Vec<(BpmnEvent,usize, Option<String>, Option<String>)>>, // Remember the events for each label <label name, (event, node id, pool, lane)>
    label_end_map: HashMap<String, (String, Option<String>)>, // Remember the join label for each branch label <label name, (join label name, optional text)>
    gateway_map: HashMap<usize, Vec<(String, Option<String>)>>, // Remember the branches for each gateway <node id, (label, optional text)>
    gateway_end_map: HashMap<usize, Vec<String>>, // Remember the join labels for each gateway <node id, <join label names>>
    gateway_types: HashMap<usize, (Token, usize, usize)>, // Remember the type of each gateway <node id, (event, line, column)>, used for error checking
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String, Token, usize, String), // Message and token that caused the error
    ExpectedJoinLabelError(String, usize, String), // Error when a join label is expected
    LexerError(LexerError),        // Propagate lexer errors
    BranchingError(Token, usize, String),        // Errors related to branching
    GatewayMatchingError((usize, usize), String),        // Error when a gateway does not match
    GatewayJoinMissingError(usize, String),      // Error when a join gateway is missings
    UnexpectedTokenAfterGoError(Token, usize, String),   // Errors related to Go tokens
    DefineNodesAfterGoError(usize, String),               // Errors related to Go nodes
    GoFromError(usize, String), // Error when a node is expected before a 'Go' token
    GoToError(usize, String), // Error when a 'Go' token has no node to join
    GenericError(String), // Generic error
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(label, token, line, highlight) => write!(f, "Unexpected token {:?} encountered {}at line {}\n{}", token, label, line, highlight),
            ParseError::ExpectedJoinLabelError(label, line, highlight) => write!(f, "Label must end with a 'J' token! Add it to label '{}' at {}\n{}", label, line, highlight),
            ParseError::LexerError(err) => write!(f, "{}", err),
            ParseError::BranchingError(token, line, highlight) => write!(f, "Unexpected token {:?} after 'X' token at line {}!\nDid you mean to do 'X ->' or 'X <-'?\n{}", token, line, highlight),
            ParseError::GatewayMatchingError(lines, highlight) => write!(f, "Gateways do not match at lines {} and {}\n{}", lines.0, lines.1, highlight),
            ParseError::GatewayJoinMissingError(line, highlight) => write!(f, "Join gateway missing for label at line {}\n{}", line, highlight),
            ParseError::UnexpectedTokenAfterGoError(token, line, highlight) => write!(f, "Unexpected token {:?} after 'G' token at line {}!\nDid you mean to do 'G ->' or 'G <-'?\n{}", token, line, highlight),
            ParseError::DefineNodesAfterGoError(line, highlight) => write!(f, "Incoming 'G' token must be used before defining nodes at line {}\n{}", line, highlight),
            ParseError::GoFromError(line, highlight) => write!(f, "Node must be defined before outgoing 'G' token at line {}\n{}", line, highlight),
            ParseError::GoToError(line, highlight) => write!(f, "Node must be defined after incoming 'G' token at line {}\n{}", line, highlight),
            ParseError::GenericError(err) => write!(f, "{}", err),
        }
    }
}

pub struct Parser<'a> {
    graph: Graph,
    lexer: Lexer<'a>,
    context: ParseContext,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self, ParseError> {
        let current_token = lexer.next_token().map_err(|err| ParseError::LexerError(err))?;
        Ok(Parser {
            graph: Graph::new(vec![], vec![]),
            lexer,
            context: ParseContext {
                last_node_id: None,
                current_pool: None,
                current_lane: None,
                current_token,
            },
        })
    }

    /// Advances to the next token
    fn advance(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token() {
            Ok(token) => {
                self.context.current_token = token;
                Ok(())
            }
            Err(err) => Err(ParseError::LexerError(err)),
        }
    }

    /// Peeks at the next token without advancing
    fn peek(&mut self) -> Result<Token, LexerError> {
        self.lexer.peek_token()
    }

    /// Parses the input and returns a graph
    pub fn parse(&mut self) -> Result<Graph, ParseError> {
        // Initialize the branching structure
        let mut branching = ParseBranching {
            label_map: HashMap::new(), // (label, events)
            label_end_map: HashMap::new(), // (label, (join label, optional text))
            gateway_map: HashMap::new(), // (node id, labels)
            gateway_end_map: HashMap::new(), // (node id, <join labels>)
            gateway_types: HashMap::new(), // (node id, event)
        };

        // Initialize the go structures
        let mut go_from_map: HashMap<usize, Vec<(String, Option<String>)>> = HashMap::new(); // (node id, <(labels, optional texts)>)
        let mut go_to_map: HashMap<String, Vec<usize>> = HashMap::new(); // (label, node ids)
        let mut go_active = false; // Flag to indicate if a go is active (outgoing)

        // Parse the input
        while self.context.current_token != Token::Eof {
            // Check if a Go is active and if it's valid
            if go_active && self.is_token_a_node(&self.context.current_token) {
                return Err(ParseError::DefineNodesAfterGoError(self.lexer.line, self.lexer.highlight_error()));
            }
            // Match the current token and parse accordingly
            let current_token = self.context.current_token.clone();
            match current_token {
                Token::Pool(label) => self.parse_pool(&label, &mut go_active),
                Token::Lane(label) => self.parse_lane(&label, &mut go_active),
                Token::Go => { self.parse_go(self.context.last_node_id, &mut go_from_map, &mut go_to_map, &mut go_active)?; continue; },
                Token::EventStart(label) => self.parse_common(BpmnEvent::Start(label)),
                Token::EventMiddle(label) => self.parse_common(BpmnEvent::Middle(label)),
                Token::EventEnd(label) => self.parse_common(BpmnEvent::End(label)),
                Token::ActivityTask(label) => self.parse_common(BpmnEvent::ActivityTask(label)),
                Token::GatewayExclusive => { self.parse_gateway(BpmnEvent::GatewayExclusive, &mut branching)?; continue; },
                Token::GatewayParallel => { self.parse_gateway(BpmnEvent::GatewayParallel, &mut branching)?; continue; },
                Token::GatewayInclusive => { self.parse_gateway(BpmnEvent::GatewayInclusive, &mut branching)?; continue; },
                Token::GatewayEvent => { self.parse_gateway(BpmnEvent::GatewayEvent, &mut branching)?; continue; },
                Token::Label(label) => self.parse_label(&mut branching, &label, &mut go_from_map, &mut go_to_map)?,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        String::new(),
                        self.context.current_token.clone(),
                        self.lexer.line,
                        self.lexer.highlight_error()
                    ));
                }
            }
            self.advance()?;
        }

        // Loop through all defined gateways
        for (gateway_from_id, labels) in branching.gateway_map {
            // Loop through all branches in the gateway
            for (label, text) in labels {
                // Check if the label defined in the gateway exists in the label_map
                let events = branching.label_map.get(&label).expect("Label not found!");
                // Use the first event to create the edge to the gateway node
                let first_event = events.get(0).expect("No events found for label");
                let node_id = self.graph.add_node(first_event.0.clone(), first_event.1.clone(), first_event.2.clone(), first_event.3.clone());
                let edge = Edge::new(gateway_from_id, node_id, text.clone());
                self.graph.add_edge(edge);
                self.context.last_node_id = Some(node_id);
                // Loop through all events in the label
                for event in &events[1..] {
                    let node_id = self.graph.add_node(event.0.clone(), event.1.clone(), event.2.clone(), event.3.clone());
                    // Check if this event is a gateway, we don't want to connect gateways to gateways
                    if self.is_event_a_gateway(&event.0) {
                        // Check if this gateway joins anywhere
                        if branching.gateway_end_map.contains_key(&node_id) {
                            // Skip adding an edge to join gateways
                            self.context.last_node_id = Some(node_id);
                            continue;
                        }
                    }
                    // Connect the current node to the previous node
                    let edge = Edge::new(self.context.last_node_id.unwrap(), node_id, None);
                    self.graph.add_edge(edge);
                    self.context.last_node_id = Some(node_id);
                }
                // Get the join label for the label
                let end_label = branching.label_end_map.get(&label);
                // Check if any gateways join this label
                let end_join_ids: Vec<usize> = branching.gateway_end_map.iter()
                    .filter_map(|(key, labels)| {
                        if labels.contains(&end_label.unwrap().0) {
                            Some(*key)
                        } else {
                            None
                        }
                    }).collect();
                // Connect the last node in the label to the joining gateways
                for end_join_id in end_join_ids {
                    // Check if the gateway types match
                    if let (Some((type_from, line_from, column_from)), Some((type_to, line_to, column_to))) = (
                        branching.gateway_types.get(&gateway_from_id),
                        branching.gateway_types.get(&end_join_id),
                    ) {
                        if type_from != type_to {
                            let error_from = self.lexer.highlight_line_error(*line_from, *column_from);
                            let error_to = self.lexer.highlight_line_error(*line_to, *column_to);
                            let error = error_from +"\n"+ &error_to;
                            return Err(ParseError::GatewayMatchingError(
                                (*line_from, *line_to),
                                error,
                            ));
                        }
                    } else {
                        return Err(ParseError::GenericError(format!(
                            "One or both gateway types are missing for IDs: {} and {}",
                            gateway_from_id, end_join_id
                        )));
                    }
                    let edge = Edge::new(self.context.last_node_id.unwrap(), end_join_id, end_label.unwrap().1.clone());
                    self.graph.add_edge(edge);
                }
            }
        }

        // Loop through all go_from_map entries `G -> label "Optional text"` 
        for (from_node_id, labels) in go_from_map {
            // Loop through all outgoing labels from the same node
            for (label, text) in labels {
                // Check if the label exists in joining nodes
                if let Some(to_node_ids) = go_to_map.get(&label) {
                    // Loop through all joining nodes
                    for to_node_id in to_node_ids {
                        // Create an edge from the current node to the joining node
                        let edge = Edge::new(from_node_id, *to_node_id, text.clone());
                        self.graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(self.graph.clone())
    }

    /// Set the current pool
    fn parse_pool(&mut self, label: &str, go_active: &mut bool) {
        self.context.current_pool = Some(label.to_string());
        self.context.current_lane = None;
        self.context.last_node_id = None;
        self.lexer.seen_start = false;
        *go_active = false;
    }

    /// Set the current lane
    fn parse_lane(&mut self, label: &str, go_active: &mut bool) {
        self.context.current_lane = Some(label.to_string());
        self.context.last_node_id = None;
        self.lexer.seen_start = false;
        *go_active = false;
    }

    /// Parse a gateway
    fn parse_gateway(
        &mut self, 
        event: BpmnEvent, 
        branching: &mut ParseBranching
    ) -> Result<(), ParseError> {
        // Assign a unique node ID to this gateway
        let node_id = self.graph.add_node_noid(event, self.context.current_pool.clone(), self.context.current_lane.clone());

        // Call the common gateway parsing logic
        return self.parse_gateway_common(node_id, branching, false)
    }
    
    /// Common logic for parsing gateways (branch or join)
    fn parse_gateway_common(
        &mut self, 
        node_id: usize, 
        branching: &mut ParseBranching,
        inside_label: bool
    ) -> Result<(), ParseError> {
        // Save the current line and error message in case of an error
        let line = self.lexer.line.clone();
        let highlighted_line = self.lexer.highlight_error();
        branching.gateway_types.insert(node_id, (self.context.current_token.clone(), line, self.lexer.column - 2));

        // Handle Branch or Join for the gateway
        self.advance()?;
        match &self.context.current_token {
            Token::Branch(_, _) => self.handle_gateway_branching(node_id, branching, inside_label)?,
            Token::JoinLabel(_) => self.handle_gateway_join(node_id, branching, inside_label)?,
            _ => return Err(ParseError::BranchingError(
                self.context.current_token.clone(),
                line,
                highlighted_line
            )),
        }

        Ok(())
    }

    // Helper to handle branching
    fn handle_gateway_branching(
        &mut self,
        node_id: usize,
        branching: &mut ParseBranching,
        inside_label: bool
    ) -> Result<(), ParseError> {
        if !inside_label {
            self.connect_nodes(node_id);
        }
        while let Token::Branch(label, text) = &self.context.current_token {
            let branch_text = if text.is_empty() { None } else { Some(text.clone()) };
            branching
                .gateway_map
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push((label.clone(), branch_text));
            self.advance()?;
        }
        Ok(())
    }

    // Helper to handle joins
    fn handle_gateway_join(
        &mut self,
        node_id: usize,
        branching: &mut ParseBranching,
        inside_label: bool,
    ) -> Result<(), ParseError> {
        if !inside_label {
            self.context.last_node_id = Some(node_id);
        }
        while let Token::JoinLabel(label) = &self.context.current_token {
            branching
                .gateway_end_map
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push(label.clone());
            self.advance()?;
        }
        Ok(())
    }

    /// Parse a branch label
    fn parse_label(
        &mut self, 
        branching: &mut ParseBranching, 
        label: &str, 
        go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, 
        go_to_map: &mut HashMap<String, Vec<usize>>, 
    ) -> Result<(), ParseError>  {
        let mut go_active_in_label = false;
        // Save the current line and error message in case of an error
        let start_line = self.lexer.line;
        let highlighted_line = self.lexer.highlight_error();
        
        // Save all events for this label
        let mut events: Vec<(BpmnEvent,usize, Option<String>, Option<String>)> = vec![]; // (event, node_id, pool, lane)
        
        // Parse all events until a join label is found
        self.advance()?;
        while !matches!(self.context.current_token, Token::Join(_,_)) && !matches!(self.context.current_token, Token::Eof) {
            // Check if a Go is active and if it's valid
            let current_token = self.context.current_token.clone();
            if go_active_in_label && self.is_token_a_node(&current_token) {
                return Err(ParseError::DefineNodesAfterGoError(self.lexer.line, self.lexer.highlight_error()));
            }
            match &current_token {
                // If the current token is a label, parse it recursively
                Token::Label(inner_label) => {
                    self.parse_label(branching, &inner_label, go_from_map, go_to_map)?;
                }
                Token::EventStart(label) => {
                    events.push(self.create_event_node(BpmnEvent::Start(label.clone()))?);
                }
                Token::EventMiddle(label) => {
                    events.push(self.create_event_node(BpmnEvent::Middle(label.clone()))?);
                }
                Token::EventEnd(label) => {
                    events.push(self.create_event_node(BpmnEvent::End(label.clone()))?);
                }
                Token::ActivityTask(label) => {
                    events.push(self.create_event_node(BpmnEvent::ActivityTask(label.clone()))?);
                }
                Token::Go => { 
                    let from_id = events.last().map(|event| event.1);
                    self.parse_go(from_id, go_from_map, go_to_map, &mut go_active_in_label)?; 
                    continue; 
                },
                Token::GatewayExclusive => {
                    self.handle_gateway_in_label(branching, &mut events)?;
                    continue;
                },
                Token::GatewayParallel => {
                    self.handle_gateway_in_label(branching, &mut events)?;
                    continue;
                },
                Token::GatewayInclusive => {
                    self.handle_gateway_in_label(branching, &mut events)?;
                    continue;
                },
                Token::GatewayEvent => {
                    self.handle_gateway_in_label(branching, &mut events)?;
                    continue;
                },
                _ => return Err(ParseError::UnexpectedToken(
                    format!("in label '{}' ", label),
                    self.context.current_token.clone(),
                    self.lexer.line,
                    self.lexer.highlight_error()
                )),
            }
            self.advance()?;
        }
        branching.label_map.insert(label.to_string(), events);
        if let Token::Join(exit_label, text) = &self.context.current_token {
            branching.label_end_map.insert(
                label.to_string(),
                (
                    exit_label.clone(), 
                    if text.is_empty() { None } else { Some(text.clone()) }
                )
            );
        } else {
            return Err(ParseError::ExpectedJoinLabelError(
                label.to_string(),
                start_line,
                highlighted_line
            ));
        }
        Ok(())
    }

    /// Create an event node
    fn create_event_node(
        &mut self,
        event: BpmnEvent,
    ) -> Result<(BpmnEvent, usize, Option<String>, Option<String>), ParseError> {
        let node_id = self.graph.next_node_id();
        Ok((
            event,
            node_id,
            self.context.current_pool.clone(),
            self.context.current_lane.clone(),
        ))
    }    

    /// Handle a gateway inside a branch
    fn handle_gateway_in_label(
        &mut self,
        branching: &mut ParseBranching,
        events: &mut Vec<(BpmnEvent, usize, Option<String>, Option<String>)>,
    ) -> Result<(), ParseError> {
        // Assign a unique node ID to this gateway
        let gateway_id = self.graph.next_node_id();
        self.context.last_node_id = Some(gateway_id);
        
        events.push((
            BpmnEvent::GatewayExclusive,
            gateway_id,
            self.context.current_pool.clone(),
            self.context.current_lane.clone(),
        ));
    
        // Store the gateway_id and parse branches without advancing
        return self.parse_gateway_common(gateway_id, branching, true);
    }

    /// Connect two nodes with an edge if needed
    fn connect_nodes(&mut self, node_id: usize) {
        if let Some(last_node_id) = self.context.last_node_id {
            let edge = Edge::new(last_node_id, node_id, None);
            self.graph.add_edge(edge);
        }
        self.context.last_node_id = Some(node_id);
    }

    /// Common function to parse an event or task
    fn parse_common(&mut self, event: BpmnEvent) {
        let node_id = self.graph.add_node_noid(event, self.context.current_pool.clone(), self.context.current_lane.clone());
        self.connect_nodes(node_id);
    }

    /// Parse a go
    fn parse_go(
        &mut self, 
        from_id: Option<usize>,
        go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, 
        go_to_map: &mut HashMap<String, Vec<usize>>, 
        go_active: &mut bool
    ) -> Result<(), ParseError> {
        // Save the current line and error message in case of an error
        let line = self.lexer.line.clone();
        let highlighted_line = self.lexer.highlight_error();

        // Check if this go is a branching go or a join go
        self.advance()?;
        match &self.context.current_token {
            Token::Branch(_, _) => {
                *go_active = true;
                self.handle_go_from(from_id, go_from_map)?;
            }
            Token::JoinLabel(_) => {
                *go_active = false;
                let next_node_id = self.graph.last_node_id + 1;
                self.handle_go_to(next_node_id, go_to_map)?;
            }
            _ => {
                return Err(ParseError::UnexpectedTokenAfterGoError(
                    self.context.current_token.clone(),
                    line,
                    highlighted_line
                ));
            } 
        }
        Ok(())
    }

    fn handle_go_from(
        &mut self,
        from_id: Option<usize>,
        go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>,
    ) -> Result<(), ParseError> {
        while let Token::Branch(label, text) = &self.context.current_token {
            // Unwrap or return an error if `from_id` is `None`
            let last_node_id = from_id.ok_or_else(|| ParseError::GoFromError(
                self.lexer.line,
                self.lexer.highlight_error()
            ))?;

            let edge_text = if text.is_empty() { None } else { Some(text.clone()) };
            go_from_map.entry(last_node_id).or_insert_with(Vec::new).push((label.clone(), edge_text));
    
            self.advance()?;
        }
    
        Ok(())
    }
    
    /// Handle parsing for a join 'Go' token
    fn handle_go_to(
        &mut self,
        next_node_id: usize,
        go_to_map: &mut HashMap<String, Vec<usize>>,
    ) -> Result<(), ParseError> {    
        // Check that a valid node type follows
        let next_token = self.peek().unwrap();
        if !self.is_token_a_node(&next_token) {
            return Err(ParseError::GoToError(
                self.lexer.line,
                self.lexer.highlight_error()
            ));
        }
    
        // Loop through all join labels and store the node IDs
        while let Token::JoinLabel(label) = &self.context.current_token {
            go_to_map.entry(
                label.clone())
                .or_insert_with(Vec::new)
                .push(next_node_id);
            self.advance()?;
        }
    
        Ok(())
    }

    fn is_token_a_node(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::EventStart(_)    | 
            Token::EventMiddle(_)   | 
            Token::EventEnd(_)      | 
            Token::ActivityTask(_)  | 
            Token::GatewayExclusive
        )
    }

    fn is_event_a_gateway(&self, token: &BpmnEvent) -> bool {
        matches!(
            token,
            BpmnEvent::GatewayExclusive | 
            BpmnEvent::GatewayParallel  | 
            BpmnEvent::GatewayInclusive | 
            BpmnEvent::GatewayEvent
        )
    }
}