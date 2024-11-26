use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::common::pool::Pool;
use std::collections::HashMap;

/// Reduces crossings in the graph by rearranging nodes within each layer.
pub fn reduce_crossings<'a>(
    pools_lanes_layers: &'a mut Vec<Pool>,
    graph: &'a Graph,
) -> &'a Vec<Pool> {
    for pool in &mut *pools_lanes_layers {
        for lane in pool.get_lanes_mut() {
            let lane_layers = lane.get_layers_mut();
            for layer in lane_layers {
                align_connected_nodes(&mut layer.get_nodes_mut(), graph);
            }
        }
    }
    pools_lanes_layers
}

/// Align nodes that are connected in the same layer to the same x-coordinate if possible.
fn align_connected_nodes(nodes_in_layer: &mut Vec<Node>, graph: &Graph) {
    let mut x_position_map: HashMap<usize, f64> = HashMap::new();
    let mut x_position = 0.0;

    for i in 0..nodes_in_layer.len() {
        let node_a = nodes_in_layer[i].id;

        let x_a = *x_position_map.entry(node_a).or_insert_with(|| {
            let pos = x_position;
            x_position += 50.0;
            pos
        });

        for j in (i + 1)..nodes_in_layer.len() {
            let node_b = nodes_in_layer[j].id;

            let has_edge = graph.edges.iter().any(|edge| {
                (edge.from == node_a && edge.to == node_b)
                    || (edge.from == node_b && edge.to == node_a)
            });

            if has_edge {
                x_position_map.insert(node_b, x_a);
            }
        }
    }
}
