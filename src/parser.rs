use std::collections::HashMap;
use crate::lexer::{Token, Lexer};
use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::{self, Graph};
use crate::common::edge::Edge;

struct ParseContext {
    last_node_id: Option<usize>,
    current_branch: Option<String>,
    current_pool: Option<String>,
    current_lane: Option<String>,
}

struct ParseBranching {
    label_map: HashMap<String, Vec<BpmnEvent>>,
    label_end_map: HashMap<String, (String, Option<String>)>,
    gateway_map: HashMap<usize, Vec<(String, Option<String>)>>,
    gateway_end_map: HashMap<usize, Vec<String>>,
}

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
        let mut context = ParseContext {
            last_node_id: None,
            current_branch: None,
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

        while self.current_token != Token::Eof {
            match &self.current_token {
                Token::Pool(label) => self.parse_pool(&mut context, label),
                Token::Lane(label) => self.parse_lane(&mut context, label),
                Token::Go => { self.parse_go(&mut graph, &mut context, &mut go_from_map, &mut go_to_map)?; continue; },
                Token::EventStart(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::Start(label.clone())),
                Token::EventMiddle(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::Middle(label.clone())),
                Token::EventEnd(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::End(label.clone())),
                Token::GatewayExclusive => { self.parse_gateway(&mut graph, &mut context, BpmnEvent::GatewayExclusive, &mut branching)?; continue; },
                // Token::Branch(label, text) => self.start_branch(&mut branches, label, text, context.last_node_id)?,
                Token::Label(label) => self.parse_label(&mut graph, &mut context, &mut branching, &label.clone())?,
                // Token::ActivityTask(label) => self.parse_task(&mut graph, &mut context, &label.clone(), &mut last_in_branch)?,
                // Token::Join(label, text) => self.record_join(&mut join_gateway, &context, label, text, &last_in_branch)?,
                // Token::JoinLabel(label) => self.process_join(&mut graph, &mut context, &label.clone(), &mut join_gateway, &mut gateway_stack)?,
                _ => return Err(format!("Unexpected token: {:?}", self.current_token)),
            }
            self.advance();
        }

        println!("{:?}", branching.label_map);
        println!("{:?}", branching.label_end_map);
        println!("{:?}", branching.gateway_map);
        println!("{:?}", branching.gateway_end_map);
        for (gateway_from_id, labels) in branching.gateway_map {
            for (label, text) in labels {
                let events = branching.label_map.get(&label).unwrap();
                let node_id = graph.add_node_noid(events.get(0).unwrap().clone(), context.current_pool.clone(), context.current_lane.clone());
                let edge = Edge::new(gateway_from_id, node_id, text.clone(), context.current_pool.clone(), context.current_lane.clone());
                graph.add_edge(edge);
                context.last_node_id = Some(node_id);
                for event in &events[1..] {
                    let node_id = graph.add_node_noid(event.clone(), context.current_pool.clone(), context.current_lane.clone());
                    let edge = Edge::new(gateway_from_id, node_id, None, context.current_pool.clone(), context.current_lane.clone());
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
                    let edge = Edge::new(context.last_node_id.unwrap(), end_join_id, end_label.unwrap().1.clone(), context.current_pool.clone(), context.current_lane.clone());
                    graph.add_edge(edge);
                }
            }
        }

        for (from_node_id, labels) in go_from_map {
            for (label, text) in labels {
                if let Some(to_node_ids) = go_to_map.get(&label) {
                    for to_node_id in to_node_ids {
                        let edge = Edge::new(from_node_id, *to_node_id, text.clone(), context.current_pool.clone(), context.current_lane.clone());
                        graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(graph)
    }

    fn parse_pool(&self, context: &mut ParseContext, label: &str) {
        context.current_pool = Some(label.to_string());
        context.current_lane = None;
        context.last_node_id = None;
    }

    fn parse_lane(&self, context: &mut ParseContext, label: &str) {
        context.current_lane = Some(label.to_string());
        context.last_node_id = None;
    }

    fn parse_gateway(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent, branching: &mut ParseBranching) -> Result<(), String> {
        let node_id = graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());
        self.advance();
        if let Token::Branch(_, _) = self.current_token {
            if context.last_node_id != None {
                let edge = Edge::new(context.last_node_id.unwrap(), node_id, None, context.current_pool.clone(), context.current_lane.clone());
                graph.add_edge(edge);
            }
            context.last_node_id = Some(node_id);
            while let Token::Branch(label, text) = &self.current_token {
                branching.gateway_map.entry(node_id).or_insert(vec![]).push((label.clone(), if text.is_empty() { None } else { Some(text.clone()) }));
                self.advance();
            }
        } else if let Token::JoinLabel(_) = self.current_token {
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

    fn parse_label(&mut self, graph: &mut Graph, context: &mut ParseContext, branching: &mut ParseBranching, label: &str) -> Result<(), String>  {
        self.advance();
        let mut events: Vec<BpmnEvent> = vec![];
        while !matches!(self.current_token, Token::Join(_,_)) {
            let current_token = self.current_token.clone();
            match &current_token {
                Token::Label(inner_label) => {
                    self.parse_label(graph, context, branching, &inner_label.clone())?;
                    // if let Some(event_list) = branching.label_map.get(inner_label) {
                    //     events.extend(event_list.clone());
                    // }
                },
                Token::EventStart(label) => events.push(BpmnEvent::Start(label.clone())),
                Token::EventMiddle(label) => events.push(BpmnEvent::Middle(label.clone())),
                Token::EventEnd(label) => events.push(BpmnEvent::End(label.clone())),
                Token::GatewayExclusive => {continue;},
                _ => (),
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

    fn parse_event(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent) {
        let node_id = graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());
        if context.last_node_id != None {
            let edge = Edge::new(context.last_node_id.unwrap(), node_id, None, context.current_pool.clone(), context.current_lane.clone());
            graph.add_edge(edge);
        }
        context.last_node_id = Some(node_id);
    }

    fn parse_go(&mut self, graph: &mut Graph, context: &mut ParseContext, go_from_map: &mut HashMap<usize, Vec<(String, Option<String>)>>, go_to_map: &mut HashMap<String, Vec<usize>>) -> Result<(), String> {
        self.advance();
        if let Token::Branch(_, _) = self.current_token {
            while let Token::Branch(label, text) = &self.current_token {
                if let Some(last_node_id) = context.last_node_id {
                    go_from_map.entry(last_node_id).or_insert(vec![]).push((label.clone(), if text.is_empty() { None } else { Some(text.clone()) }));
                } else {
                    return Err(format!("Expected a node before branch label! Current token: {:?}", self.current_token));
                }
                self.advance();
            }
        } else if let Token::JoinLabel(_) = self.current_token {
            let next_node_id = graph.nodes.len() + 1;
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