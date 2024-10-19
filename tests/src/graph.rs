use crate::Edge;
use crate::Node;

pub struct Graph {
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
}