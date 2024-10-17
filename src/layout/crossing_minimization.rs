use crate::layout::graph::Graph;
use std::collections::HashMap;

/// Reduces crossings in the graph by rearranging nodes within each layer.
pub fn reduce_crossings(graph: &mut Graph, layers: &Vec<(usize, i32)>) -> Vec<(usize, i32)> {
    let mut layer_map: HashMap<i32, Vec<usize>> = HashMap::new();

    // Group nodes by their layers
    for (node_id, layer) in layers {
        layer_map.entry(*layer).or_insert(vec![]).push(*node_id);
    }

    // For each layer except the first, perform barycenter-based sorting
    for current_layer in 1..layer_map.len() {
        let prev_layer = current_layer as i32 - 1;
        let nodes_in_prev_layer = layer_map.get(&(prev_layer as i32)).unwrap().clone();
        let nodes_in_current_layer = layer_map.get_mut(&(current_layer as i32)).unwrap();

        // Calculate barycenter values for nodes in the current layer
        let mut barycenters: Vec<(usize, f64)> = nodes_in_current_layer
            .iter()
            .map(|&node_id| {
                let neighbors_in_prev_layer: Vec<usize> = graph
                    .edges
                    .iter()
                    .filter(|edge| edge.to == node_id && nodes_in_prev_layer.contains(&edge.from))
                    .map(|edge| edge.from)
                    .collect();

                // Calculate the barycenter as the average position of the neighbors in the previous layer
                let sum_positions: f64 = neighbors_in_prev_layer
                    .iter()
                    .map(|&neighbor_id| nodes_in_prev_layer.iter().position(|&id| id == neighbor_id).unwrap() as f64)
                    .sum();

                let barycenter_value = if neighbors_in_prev_layer.len() > 0 {
                    sum_positions / neighbors_in_prev_layer.len() as f64
                } else {
                    node_id as f64 // No neighbors, use the node ID as fallback
                };

                (node_id, barycenter_value)
            })
            .collect();

        // Sort the current layer nodes by their barycenter values
        barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        *nodes_in_current_layer = barycenters.into_iter().map(|(node_id, _)| node_id).collect();
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
