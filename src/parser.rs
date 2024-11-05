use std::collections::HashMap;
use crate::lexer::{Token, Lexer};
use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::Graph;
use crate::common::edge::Edge;

struct ParseContext {
    last_node_id: Option<usize>,
    current_branch: Option<String>,
    current_pool: Option<String>,
    current_lane: Option<String>,
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

        let mut branches = HashMap::new();
        let mut last_in_branch = HashMap::new();
        let mut join_gateway = HashMap::new();
        let mut gateway_stack = Vec::new();

        while self.current_token != Token::Eof {
            match &self.current_token {
                Token::Pool(label) => self.parse_pool(&mut context, label),
                Token::Lane(label) => self.parse_lane(&mut context, label),
                Token::EventStart(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::Start(label.clone())),
                Token::EventMiddle(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::Middle(label.clone())),
                Token::EventEnd(label) => self.parse_event(&mut graph, &mut context, BpmnEvent::End(label.clone())),
                Token::GatewayExclusive => self.start_gateway(&mut graph, &mut context, &mut gateway_stack),
                Token::Branch(label, text) => self.start_branch(&mut branches, label, text, context.last_node_id)?,
                Token::Label(label) => self.set_current_branch(&mut context, label, &branches, &mut last_in_branch)?,
                Token::ActivityTask(label) => self.parse_task(&mut graph, &mut context, &label.clone(), &mut last_in_branch)?,
                Token::Join(label, text) => self.record_join(&mut join_gateway, &context, label, text, &last_in_branch)?,
                Token::JoinLabel(label) => self.process_join(&mut graph, &mut context, &label.clone(), &mut join_gateway, &mut gateway_stack)?,
                _ => return Err(format!("Unexpected token: {:?}", self.current_token)),
            }
            self.advance();
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

    fn parse_event(&mut self, graph: &mut Graph, context: &mut ParseContext, event: BpmnEvent) {
        let node_id = graph.add_node_noid(event, context.current_pool.clone(), context.current_lane.clone());
        self.add_edge(graph, context.last_node_id, node_id, None, &context);
        context.last_node_id = Some(node_id);
    }

    fn parse_task(
        &mut self,
        graph: &mut Graph,
        context: &mut ParseContext,
        label: &str,
        last_in_branch: &mut HashMap<String, (usize, String)>,
    ) -> Result<(), String> {
        let node_id = graph.add_node_noid(BpmnEvent::ActivityTask(label.to_string()), context.current_pool.clone(), context.current_lane.clone());
        
        if let Some(branch_label) = &context.current_branch {
            if let Some((last_node_id, text)) = last_in_branch.get(branch_label) {
                let edge_text = if text.is_empty() { None } else { Some(text.clone()) };
                graph.add_edge(Edge::new(*last_node_id, node_id, edge_text, context.current_pool.clone(), context.current_lane.clone()));
                last_in_branch.insert(branch_label.clone(), (node_id, String::new())); // Update last node in branch
            } else {
                return Err(format!("No start node found for branch '{}'", branch_label));
            }
        } else if let Some(prev_id) = context.last_node_id {
            graph.add_edge(Edge::new(prev_id, node_id, None, context.current_pool.clone(), context.current_lane.clone()));
        }
        
        context.last_node_id = Some(node_id);
        Ok(())
    }

    fn add_edge(
        &self,
        graph: &mut Graph,
        from: Option<usize>,
        to: usize,
        text: Option<String>,
        context: &ParseContext,
    ) {
        if let Some(prev_id) = from {
            graph.add_edge(Edge::new(prev_id, to, text, context.current_pool.clone(), context.current_lane.clone()));
        }
    }

    fn start_gateway(
        &mut self,
        graph: &mut Graph,
        context: &mut ParseContext,
        gateway_stack: &mut Vec<Vec<String>>,
    ) {
        let node_id = graph.add_node_noid(BpmnEvent::GatewayExclusive, context.current_pool.clone(), context.current_lane.clone());
        self.add_edge(graph, context.last_node_id, node_id, None, context);
        context.last_node_id = Some(node_id);
        gateway_stack.push(Vec::new()); // Start a new set of branches for the gateway
    }

    fn set_current_branch(
        &self,
        context: &mut ParseContext,
        label: &str,
        branches: &HashMap<String, (usize, String)>,
        last_in_branch: &mut HashMap<String, (usize, String)>,
    ) -> Result<(), String> {
        context.current_branch = Some(label.to_string());
        if let Some((start_node_id, branch_text)) = branches.get(label) {
            last_in_branch.insert(label.to_string(), (*start_node_id, branch_text.clone()));
            Ok(())
        } else {
            Err(format!("No start node found for branch '{}'", label))
        }
    }

    fn record_join(
        &self,
        join_gateway: &mut HashMap<String, Vec<(usize, String)>>,
        context: &ParseContext,
        label: &str,
        text: &str,
        last_in_branch: &HashMap<String, (usize, String)>,
    ) -> Result<(), String> {
        if let Some(branch_label) = &context.current_branch {
            if let Some((last_node_id, _)) = last_in_branch.get(branch_label) {
                join_gateway
                    .entry(label.to_string())
                    .or_insert_with(Vec::new)
                    .push((*last_node_id, text.to_string()));
                Ok(())
            } else {
                Err(format!("No last node found in branch '{}' for join '{}'", branch_label, label))
            }
        } else {
            Err(format!("No active branch found for join '{}'", label))
        }
    }

    fn process_join(
        &mut self,
        graph: &mut Graph,
        context: &mut ParseContext,
        label: &str,
        join_gateway: &mut HashMap<String, Vec<(usize, String)>>,
        gateway_stack: &mut Vec<Vec<String>>,
    ) -> Result<(), String> {
        let node_id = graph.add_node_noid(BpmnEvent::GatewayJoin(label.to_string()), context.current_pool.clone(), context.current_lane.clone());
        if let Some(nodes_to_join) = join_gateway.remove(label) {
            for (prev_id, text) in nodes_to_join {
                let edge_text = if text.is_empty() { None } else { Some(text.clone()) };
                graph.add_edge(Edge::new(prev_id, node_id, edge_text, context.current_pool.clone(), context.current_lane.clone()));
            }
        } else {
            return Err(format!("No join recorded for label '{}'", label));
        }
        context.last_node_id = Some(node_id);
        
        Ok(())
    }

    fn start_branch(
        &self,
        branches: &mut HashMap<String, (usize, String)>,
        label: &str,
        text: &str,
        last_node_id: Option<usize>,
    ) -> Result<(), String> {
        if let Some(prev_id) = last_node_id {
            branches.insert(label.to_string(), (prev_id, text.to_string()));
            Ok(())
        } else {
            Err(format!("No previous node found for starting branch '{}'", label))
        }
    }
}