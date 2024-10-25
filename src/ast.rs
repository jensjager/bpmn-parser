use crate::common::graph;
// src/ast.rs
use crate::common::{edge::Edge, graph::Graph, node::Node};
use crate::common::bpmn_event::BpmnEvent;

pub struct Ast {
    graph: Graph
}

impl Ast {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(vec![], vec![])
        }
    }

    pub fn print_graph(&self) {
        for node in &self.graph.nodes {
            if let Some(event) = &node.event {
                match event {
                    BpmnEvent::Start(label) => println!("  Start Event: {} (ID: {})", label, node.id),
                    BpmnEvent::Middle(label) => println!("  Middle Event: {} (ID: {})", label, node.id),
                    BpmnEvent::End(label) => println!("  End Event: {} (ID: {})", label, node.id),
                    BpmnEvent::GatewayExclusive => println!("  GatewayExclusive Event (ID: {})", node.id),
                    BpmnEvent::ActivityTask(label) => println!("  ActivityTask: {} (ID: {})", label, node.id),
                    BpmnEvent::GatewayJoin(label) => println!("  GatewayJoin Event: {} (ID: {})", label, node.id),
                }
            } else {
                println!("  No Event (ID: {})", node.id);
            }
        }

        println!("Edges:");
        for edge in &self.graph.edges {
            if let Some(text) = &edge.text {
                println!("  From Node {} to Node {}: '{}'", edge.from, edge.to, text);
            } else {
                println!("  From Node {} to Node {}", edge.from, edge.to);
            }
        }
    }

}
