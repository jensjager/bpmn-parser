// graph.rs
use crate::common::node::Node;
use crate::common::edge::Edge;
use crate::common::bpmn_event::BpmnEvent;

/// Represents a graph consisting of nodes and edges.
pub struct Graph {
    pub nodes: Vec<Node>,  // Nodes
    pub edges: Vec<Edge>,  // Edges
    last_node_id: usize,   // Last used node ID
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Graph { nodes, edges, last_node_id: 0 }
    }

    pub fn add_node_noid(&mut self, bpmn_event: BpmnEvent, pool: Option<String>, lane: Option<String>) -> usize {
        let new_node = Node::new(self.last_node_id + 1, None, None, Some(bpmn_event), pool, lane);
        
        self.nodes.push(new_node);

        self.last_node_id += 1;
        self.last_node_id
    }

    pub fn add_node(&mut self, bpmn_event: BpmnEvent, id: usize, pool: Option<String>, lane: Option<String>) -> usize {
        let new_node = Node::new(id, None, None, Some(bpmn_event), pool, lane);
        
        self.nodes.push(new_node);

        id
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    // Get the next node ID.
    pub fn next_node_id(&mut self) -> usize {
        self.last_node_id += 1;  // Increment the last used ID
        self.last_node_id         // Return the new ID
    }
}
