use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::graph::NodeId;
use crate::common::node::Node;
use std::collections::HashMap;

/// Reduces crossings in the graph by rearranging nodes within each layer.
pub fn reduce_crossings(_: &mut Graph) {
    //for pool in &mut graph.pools {
        //for lane in &mut pool.lanes {
            //align_connected_nodes(&mut lane.layers, &graph.edges);
        //}
    //}
}

/// Align nodes that are connected and share the same layer by their layer ID
#[allow(unused)]
fn align_connected_nodes(nodes: &mut Vec<Node>, edges: &[Edge]) {
    let mut x_position_map: HashMap<NodeId, f64> = HashMap::new();
    let mut x_position = 0.0;

    // Group nodes by layer
    let mut layer_groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        layer_groups
            .entry(node.layer_id.unwrap_or(0))
            .or_default()
            .push(idx);
    }

    // Process each layer group
    for (_layer_id, indices) in layer_groups {
        for &i in &indices {
            let node_a = nodes[i].id;

            let x_a = *x_position_map.entry(node_a).or_insert_with(|| {
                let pos = x_position;
                x_position += 50.0;
                pos
            });

            for &j in indices.iter().filter(|&&j| j > i) {
                let node_b = nodes[j].id;

                let has_edge = edges.iter().any(|edge| {
                    (edge.from == node_a && edge.to == node_b)
                        || (edge.from == node_b && edge.to == node_a)
                });

                if has_edge {
                    x_position_map.insert(node_b, x_a);
                }
            }
        }
    }

    // Update node positions
    for node in nodes {
        if let Some(&x) = x_position_map.get(&node.id) {
            node.x = Some(x);
        }
    }
}
