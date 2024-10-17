use crate::layout::node::Node;
use crate::layout::edge::Edge;
use std::collections::HashSet;

/// Represents a graph consisting of nodes and edges.
pub struct Graph {
    pub nodes: Vec<Node>,  // Nodes
    pub edges: Vec<Edge>,  // Edges
}

impl Graph {
    /// Eemaldab tagasiservad ja muudab graafi suunatud atsükliliseks graafiks.
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
    pub fn eliminate_back_edges(&mut self) {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut back_edges = Vec::new();

        // Abifunktsioon DFS-i jaoks
        fn dfs(
            graph: &Graph,
            node_id: usize,
            visited: &mut HashSet<usize>,
            stack: &mut HashSet<usize>,
            back_edges: &mut Vec<Edge>,
        ) {
            visited.insert(node_id);
            stack.insert(node_id);

            for edge in graph.edges.iter().filter(|e| e.from == node_id) {
                if stack.contains(&edge.to) {
                    // Leitud tagasiserv, lisa see tagasiservade nimekirja
                    back_edges.push(Edge::new(edge.from, edge.to));
                } else if !visited.contains(&edge.to) {
                    dfs(graph, edge.to, visited, stack, back_edges);
                }
            }
            stack.remove(&node_id);
        }

        // Käivitame DFS-i iga sõlme jaoks
        for node in &self.nodes {
            if !visited.contains(&node.id) {
                dfs(self, node.id, &mut visited, &mut stack, &mut back_edges);
            }
        }

        // Pöörame või eemaldame tagasiservad, et graaf muutuks atsükliliseks
        for edge in back_edges {
            self.reverse_or_remove_edge(edge);
        }
    }

    /// Abifunktsioon serva ümberpööramiseks või eemaldamiseks graafis.
    fn reverse_or_remove_edge(&mut self, edge: Edge) {
        // Leiame ja eemaldame originaalse serva
        if let Some(index) = self.edges.iter().position(|e| e.from == edge.from && e.to == edge.to) {
            self.edges.remove(index);
        }

        // Lisame vastupidise serva ainult siis kui see veel ei eksisteeri
        if !self.edges.iter().any(|e| e.from == edge.to && e.to == edge.from) {
            self.edges.push(Edge::new(edge.to, edge.from));
        }
    }
}
