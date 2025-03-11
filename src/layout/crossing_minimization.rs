use crate::common::graph::Graph;
use crate::common::graph::NodeId;
use crate::common::node::Node;
use std::cmp::Ordering::Equal;
use std::collections::HashMap;

/// Reduces crossings in the graph by rearranging nodes within each layer.
/// Align nodes that are connected and share the same layer by their layer ID
pub fn reduce_crossings(graph: &mut Graph) {
    for pool in graph.pools.iter_mut() {
        for lane in pool.lanes.iter_mut() {
            lane.sort_nodes_by_layer_id(&graph.nodes);
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        let mut node_averages: HashMap<NodeId, f64> = HashMap::new();
        for pool in graph.pools.iter() {
            for i in 1..pool.lanes.len() {
                for &node_id in &pool.lanes[i].nodes {
                    let average = find_average(node_id, graph).unwrap_or(0.0);
                    node_averages.insert(node_id, average);
                }
            }
        }

        for pool in graph.pools.iter_mut() {
            for (lane_index, lane) in pool.lanes.iter_mut().enumerate() {
                if lane_index == 0 {
                    continue;
                }

                let old_order = lane.nodes.clone();

                lane.nodes.sort_by(|a, b| {
                    let node_a = &graph.nodes[a.0];
                    let node_b = &graph.nodes[b.0];
                    match node_a.layer_id.cmp(&node_b.layer_id) {
                        Equal => {
                            let avg_a = node_averages.get(a).unwrap_or(&0.0);
                            let avg_b = node_averages.get(b).unwrap_or(&0.0);
                            avg_a.partial_cmp(avg_b).unwrap_or(Equal)
                        }
                        ord => ord,
                    }
                });

                if lane.nodes != old_order {
                    changed = true;
                }
            }
        }
    }
}

fn find_average(node_id: NodeId, graph: &Graph) -> Option<f64> {
    let mut sum: usize = 0;
    let mut count: usize = 0;
    let to_node_lane: &Option<String> = &graph.nodes[node_id.0].lane;

    for edge in graph.edges.iter() {
        if edge.to == node_id {
            let from_node = &graph.nodes[edge.from.0];
            count += 1;
            let (pos, passed_to_node_lane) =
                get_node_position_in_layer(graph, from_node, to_node_lane);
            if pos == None {
                // TODO throw error
            }
            if from_node.lane == *to_node_lane {
                sum += pos.unwrap_or(0);
            } else if passed_to_node_lane {
                // The node is connected to a node from another lane
                return Some(f64::MAX); // Move node to the bottom of the layer
            } else {
                return Some(f64::MIN); // Move node to the top of the layer
            }
        }
    }

    if count > 0 {
        Some(sum as f64 / count as f64)
    } else {
        None
    }
}

pub fn get_node_position_in_layer(
    graph: &Graph,
    node: &Node,
    to_node_lane: &Option<String>,
) -> (Option<usize>, bool) {
    let mut passed_to_node_lane = false;
    for pool in &graph.pools {
        if pool.pool_name == node.pool {
            for lane in &pool.lanes {
                if lane.lane == *to_node_lane {
                    passed_to_node_lane = true;
                }
                if lane.lane == node.lane {
                    for i in 0..lane.nodes.len() {
                        let cur_node = &graph.nodes[lane.nodes[i].0];
                        if cur_node.layer_id == node.layer_id {
                            let mut pos = 0;
                            for j in i..lane.nodes.len() {
                                if lane.nodes[j] == node.id {
                                    pos += 1;
                                    return (Some(pos), passed_to_node_lane);
                                }
                                pos += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    (None, passed_to_node_lane)
}
