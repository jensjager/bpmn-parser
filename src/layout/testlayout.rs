use crate::common::node::Node;
use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::layout::solve_layer_assignment::solve_layer_assignment;
use crate::layout::crossing_minimization::reduce_crossings;
use crate::layout::assign_bend_points::assign_bend_points;
use crate::layout::node_positioning::assign_xy_to_nodes;

pub fn run_test_layout() {
    let mut graph = Graph::new(vec![], vec![]);

    // Lisame sõlmed
    graph.add_node(Node::new(1, None, None, None)); // Start event
    graph.add_node(Node::new(2, None, None, None)); // Task 1
    graph.add_node(Node::new(3, None, None, None)); // Gateway
    graph.add_node(Node::new(4, None, None, None)); // Task 2
    graph.add_node(Node::new(5, None, None, None)); // Task 3
    graph.add_node(Node::new(6, None, None, None)); // End event

    // Lisame servad (ühendused)
    graph.add_edge(Edge::new(1, 2, None));
    graph.add_edge(Edge::new(2, 3, None));
    graph.add_edge(Edge::new(2, 4, None));
    graph.add_edge(Edge::new(4, 5, None));
    graph.add_edge(Edge::new(5, 6, None));

    // Kihi määramine
    let layers = solve_layer_assignment(&graph);

    println!("\nLayer assignment after back-edge elimination:");
    for (node_id, layer) in &layers {
        println!("Node {}: Layer {}", node_id, layer);
    }

    // Ristumiste minimeerimine
    let new_layers = reduce_crossings(&mut graph, &layers);

    // X-Y määramine
    assign_xy_to_nodes(&mut graph, &new_layers);

    println!("\nNode positions after X-Y assignment:");
    for node in &graph.nodes {
        println!("Node {}: x = {:?}, y = {:?}", node.id, node.x, node.y);
    }

    // Servade painutamine
    assign_bend_points(&mut graph);

    println!("\nEdge bend points after X-Y assignment:");
    for edge in &graph.edges {
        println!("Edge from {} to {} bend points: {:?}", edge.from, edge.to, edge.bend_points);
    }
}
