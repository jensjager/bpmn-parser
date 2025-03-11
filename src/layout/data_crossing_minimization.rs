use std::collections::HashMap;

use crate::common::graph::{DataNodeId, Graph};
use crate::layout::crossing_minimization::get_node_position_in_layer;

pub fn reduce_data_crossings(graph: &mut Graph) {
    graph.sort_data_nodes();

    let mut changed = true;
    while changed {
        changed = false;

        let mut dn_averages: HashMap<DataNodeId, f64> = HashMap::new();
        for i in 0..graph.data_nodes.len() {
            let dn_id = graph.data_nodes[i].id;
            let average = find_average(dn_id, graph).unwrap_or(0.0);
            dn_averages.insert(dn_id, average);
        }

        let old_order: Vec<DataNodeId> = graph.data_nodes.iter().map(|dn| dn.id.clone()).collect();
        graph.sort_data_nodes_by_average(&dn_averages);
        let new_order: Vec<DataNodeId> = graph.data_nodes.iter().map(|dn| dn.id.clone()).collect();

        if new_order != old_order {
            changed = true;
        }
    }
}

fn find_average(dn_id: DataNodeId, graph: &Graph) -> Option<f64> {
    let mut sum: usize = 0;
    let mut count: usize = 0;
    let dn_lane: &Option<String> = &graph.data_nodes[dn_id.0].lane;

    for data_edge in graph.data_edges.iter() {
        if data_edge.from == dn_id {
            let to_node = &graph.nodes[data_edge.to.0];
            count += 1;
            let (pos, passed_dn_lane) = get_node_position_in_layer(graph, to_node, dn_lane);
            if pos == None {
                // TODO throw error
            }
            if to_node.lane == *dn_lane {
                sum += pos.unwrap_or(0);
            } else if passed_dn_lane {
                // The data_node is connected to a node from another lane
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
