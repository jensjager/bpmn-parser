use crate::layout::node::Node;
use crate::layout::edge::Edge;
use crate::layout::eliminate_back_edges::eliminate_back_edges; // Import the new function
use std::collections::HashSet;

/// Represents a graph consisting of nodes and edges.
pub struct Graph {
    pub nodes: Vec<Node>,  // Nodes
    pub edges: Vec<Edge>,  // Edges
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Graph { nodes, edges }
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Removes back edges and makes the graph acyclic by calling the helper function from `back_edges.rs`.
    pub fn eliminate_back_edges(&mut self) {
        eliminate_back_edges(self);
    }
}
