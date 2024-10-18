use crate::common::node::Node;
use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::layout::solve_layer_assignment::solve_layer_assignment;
use crate::layout::crossing_minimization::reduce_crossings;

pub fn run_test_layout() {
    let mut graph = Graph::new(vec![], vec![]);

    // Add nodes
    graph.add_node(Node::with_default_event(1, None));
    graph.add_node(Node::with_default_event(2, None));
    graph.add_node(Node::with_default_event(3, None));
    graph.add_node(Node::with_default_event(4, None));
    graph.add_node(Node::with_default_event(5, None));
    graph.add_node(Node::with_default_event(6, None));

    // Add edges that form multiple cycles: 1 -> 2 -> 3 -> 4 -> 1 and 3 -> 5 -> 6 -> 3
    graph.add_edge(Edge::with_default_text(1, 2));
    graph.add_edge(Edge::with_default_text(2, 3));
    graph.add_edge(Edge::with_default_text(3, 4));
    graph.add_edge(Edge::with_default_text(4, 1)); // Cycle 1-2-3-4-1

    graph.add_edge(Edge::with_default_text(3, 5));
    graph.add_edge(Edge::with_default_text(5, 6));
    graph.add_edge(Edge::with_default_text(6, 3)); // Cycle 3-5-6-3

    graph.add_edge(Edge::with_default_text(5, 2)); 

    println!("Before back-edge elimination:");
    for edge in &graph.edges {
        println!("Edge from {} to {}", edge.from, edge.to);
    }

    // Perform back-edge elimination
    graph.eliminate_back_edges();

    println!("After back-edge elimination:");
    for edge in &graph.edges {
        println!("Edge from {} to {}", edge.from, edge.to);
    }

    // Second, test layer assignment
    let layers = solve_layer_assignment(&graph);

    println!("\nLayer assignment after back-edge elimination:");
    for (node_id, layer) in &layers {
        println!("Node {}: Layer {}", node_id, layer);
    }

    // Third, perform crossing minimization
    let new_layers = reduce_crossings(&mut graph, &layers);

    println!("\nLayer assignment after crossing minimization:");
    for (node_id, layer) in new_layers {
        println!("Node {}: Layer {}", node_id, layer);
    }
}
