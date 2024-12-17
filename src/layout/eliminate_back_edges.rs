use crate::common::graph::Graph;
use crate::common::edge::Edge;
use std::collections::HashSet;

pub fn eliminate_back_edges(graph: &mut Graph) {
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
                back_edges.push(Edge::new(edge.from, edge.to, None));
            } else if !visited.contains(&edge.to) {
                dfs(graph, edge.to, visited, stack, back_edges);
            }
        }
        stack.remove(&node_id);
    }

    // Käivitame DFS-i iga sõlme jaoks
    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            dfs(graph, node.id, &mut visited, &mut stack, &mut back_edges);
        }
    }

    // Pöörame või eemaldame tagasiservad, et graaf muutuks atsükliliseks
    for edge in back_edges {
        reverse_or_remove_edge(graph, edge);
    }
}

/// Abifunktsioon serva ümberpööramiseks või eemaldamiseks graafis.
fn reverse_or_remove_edge(graph: &mut Graph, edge: Edge) {
    // Leiame ja eemaldame originaalse serva
    if let Some(index) = graph.edges.iter().position(|e| e.from == edge.from && e.to == edge.to) {
        graph.edges.remove(index);
    }

    // Lisame vastupidise serva ainult siis kui see veel ei eksisteeri
    if !graph.edges.iter().any(|e| e.from == edge.to && e.to == edge.from) {
        graph.edges.push(Edge::new(edge.to, edge.from, None));
    }
}
