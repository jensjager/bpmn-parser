use crate::common::graph::Graph;
use std::collections::HashMap;

/// Reduces crossings in the graph by rearranging nodes within each layer.
pub fn reduce_crossings(graph: &Graph, layers: &Vec<(usize, i32)>) -> Vec<(usize, i32)> {
    let mut layer_map: HashMap<i32, Vec<usize>> = HashMap::new();

    // Group nodes by their layers
    for (node_id, layer) in layers {
        layer_map.entry(*layer).or_insert(vec![]).push(*node_id);
    }

    // For each layer, perform alignment of connected nodes to avoid crossings
    for (_, nodes_in_layer) in layer_map.iter_mut() {
        align_connected_nodes(graph, nodes_in_layer);
    }

    // Flatten the map back to a vector and return (but keep layers intact)
    let mut new_layers = vec![];
    for (layer, nodes) in layer_map {
        for node in nodes {
            new_layers.push((node, layer));
        }
    }

    new_layers
}

/// Align nodes that are connected in the same layer to the same x-coordinate if possible.
fn align_connected_nodes(graph: &Graph, nodes_in_layer: &mut Vec<usize>) {
    let mut x_position_map: HashMap<usize, f64> = HashMap::new();
    let mut x_position = 0.0;

    // Traverse each node in the current layer
    for i in 0..nodes_in_layer.len() {
        let node_a = nodes_in_layer[i];

        // If node A already has an X position, use it; otherwise, assign a new X position
        let x_a = *x_position_map.entry(node_a).or_insert_with(|| {
            let pos = x_position;
            x_position += 50.0; // Defineerime sõlme vahelise kauguse
            pos
        });

        // Check if other nodes in the layer are connected to node A and align their x-coordinate
        for j in (i + 1)..nodes_in_layer.len() {
            let node_b = nodes_in_layer[j];

            let has_edge = graph
                .edges
                .iter()
                .any(|edge| (edge.from == node_a && edge.to == node_b) || (edge.from == node_b && edge.to == node_a));

            if has_edge {
                // Align node B to the same X-coordinate as node A
                x_position_map.insert(node_b, x_a);
            }
        }
    }

    // Update the nodes in the layer with their aligned X positions
    // for node in nodes_in_layer.iter_mut() {
    //     if let Some(&_x) = x_position_map.get(node) {
    //         println!("Paigutan sõlme {} X-koordinaadile: {}", node, x);
    //     }
    // }
}
