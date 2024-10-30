// graph.rs
use crate::common::node::Node;
use crate::common::edge::Edge;
use crate::common::bpmn_event::BpmnEvent;

/// Represents a graph consisting of nodes and edges.
pub struct Graph {
    pub nodes: Vec<Node>,  // Nodes
    pub edges: Vec<Edge>,  // Edges
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Graph { nodes, edges }
    }

    pub fn add_node_noid(&mut self, bpmn_event: BpmnEvent, pool: Option<String>, lane: Option<String>) -> usize {
        let id = if let Some(last_node) = self.nodes.last() {
            last_node.id
        } else {
            0
        };

        let new_node = Node::new(id + 1, None, None, Some(bpmn_event), pool, lane);
        
        self.nodes.push(new_node);

        id + 1
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
}
