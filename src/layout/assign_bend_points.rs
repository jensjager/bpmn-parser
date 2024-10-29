use crate::common::graph::Graph;
use crate::to_xml::get_node_size;

/// Määrab servadele painutuspunktid ainult juhul, kui see on vajalik.
pub fn assign_bend_points(graph: &mut Graph) {
    for edge in &mut graph.edges {
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from).unwrap();
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to).unwrap();

        // Kasuta funktsiooni `get_node_size`, et määrata sõlmede suurused
        let (from_width, from_height) = get_node_size(from_node);
        let (to_width, to_height) = get_node_size(to_node);

        let mut bend_points = vec![];

        let (edge_from_x, edge_from_y) = if from_node.x.unwrap() < to_node.x.unwrap() {
            (from_node.x.unwrap() + from_width as f64, from_node.y.unwrap() + from_height as f64 / 2.0)
        } else if from_node.x.unwrap() > to_node.x.unwrap() {
            (from_node.x.unwrap(), from_node.y.unwrap() + from_height as f64 / 2.0)
        } else if from_node.y.unwrap() < to_node.y.unwrap() {
            (from_node.x.unwrap() + from_width as f64 / 2.0, from_node.y.unwrap() + from_height as f64)
        } else {
            (from_node.x.unwrap() + from_width as f64 / 2.0, from_node.y.unwrap())
        };

        let (edge_to_x, edge_to_y) = if from_node.x.unwrap() < to_node.x.unwrap() {
            (to_node.x.unwrap(), to_node.y.unwrap() + to_height as f64 / 2.0)
        } else if from_node.x.unwrap() > to_node.x.unwrap() {
            (to_node.x.unwrap() + to_width as f64, to_node.y.unwrap() + to_height as f64 / 2.0)
        } else if from_node.y.unwrap() < to_node.y.unwrap() {
            (to_node.x.unwrap() + to_width as f64 / 2.0, to_node.y.unwrap())
        } else {
            (to_node.x.unwrap() + to_width as f64 / 2.0, to_node.y.unwrap() + to_height as f64)
        };

        // Lisame painutuspunktid ainult siis, kui need on vajalikud
        if from_node.x != to_node.x && from_node.y != to_node.y {
            if (from_node.x.unwrap() - to_node.x.unwrap()).abs() > (from_node.y.unwrap() - to_node.y.unwrap()).abs() {
                bend_points.push((to_node.x.unwrap(), from_node.y.unwrap()));
            } else {
                bend_points.push((from_node.x.unwrap(), to_node.y.unwrap()));
            }
        }

        edge.bend_points = bend_points;
    }
}
