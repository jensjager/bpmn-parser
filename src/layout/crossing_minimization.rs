use crate::common::graph::Graph;
use crate::common::graph::NodeId;
use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use std::dbg;

/// Reduces crossings in the graph by rearranging nodes within each layer.
/// Align nodes that are connected and share the same layer by their layer ID
pub fn reduce_crossings(graph: &mut Graph) {
    for pool in graph.pools.iter_mut() {
        for lane in pool.lanes.iter_mut() {
            let mut change_happened = true;
            while change_happened {
                change_happened = false;
                let mut nodes_with_averages: HashMap<usize, Vec<NodeId>> = HashMap::new();

                for nodeid in lane.nodes.iter() {
                    for edge in graph.edges.iter() {
                        if edge.from == *nodeid {
                            let to_node = graph.nodes.get(edge.to.0).unwrap();
                            // TODO temp
                            if to_node.lane != lane.lane {
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}
