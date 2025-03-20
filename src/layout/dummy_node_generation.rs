use std::collections::HashMap;

use crate::common::graph::{EdgeId, Graph, NodeId};

pub fn generate_dummy_nodes(graph: &mut Graph) {
    let mut dummy_nodes: HashMap<
        (NodeId, NodeId),
        Vec<(Option<String>, Option<String>, Option<usize>)>,
    > = HashMap::new();
    for edge in graph.edges.iter() {
        let from_node = &graph.nodes[edge.from.0];
        let to_node = &graph.nodes[edge.to.0];

        let steps_to_middle = from_node
            .layer_id
            .unwrap()
            .abs_diff(to_node.layer_id.unwrap())
            / 2;

        let (start_layer, end_layer, cur_pool, start_lane, end_lane): (
            usize,
            usize,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = if from_node.layer_id < to_node.layer_id {
            (
                from_node.layer_id.unwrap(),
                to_node.layer_id.unwrap(),
                from_node.pool.clone(),
                from_node.lane.clone(),
                to_node.lane.clone(),
            )
        } else {
            (
                to_node.layer_id.unwrap(),
                from_node.layer_id.unwrap(),
                to_node.pool.clone(),
                to_node.lane.clone(),
                from_node.lane.clone(),
            )
        };
        if from_node.pool == to_node.pool && from_node.lane != to_node.lane {
            dummy_nodes.insert((from_node.id, to_node.id), vec![]);
            for i in start_layer + 1..start_layer + steps_to_middle {
                dummy_nodes
                    .get_mut(&(from_node.id, to_node.id))
                    .unwrap()
                    .push((cur_pool.clone(), start_lane.clone(), Some(i)));
            }
            for i in start_layer + steps_to_middle + 1..end_layer {
                dummy_nodes
                    .get_mut(&(from_node.id, to_node.id))
                    .unwrap()
                    .push((cur_pool.clone(), end_lane.clone(), Some(i)));
            }
            for pool in graph.pools.iter() {
                if pool.pool_name == cur_pool {
                    let mut encounters: usize = 0;
                    for lane in pool.lanes.iter() {
                        if lane.lane == start_lane || lane.lane == end_lane {
                            encounters += 1;
                        }
                        if encounters == 1 {
                            dummy_nodes
                                .get_mut(&(from_node.id, to_node.id))
                                .unwrap()
                                .push((
                                    cur_pool.clone(),
                                    lane.lane.clone(),
                                    Some(start_layer + steps_to_middle),
                                ));
                        }
                        if encounters == 2 {
                            break;
                        }
                    }
                    break;
                }
            }
        } else if start_layer + 1 != end_layer {
            dummy_nodes.insert((from_node.id, to_node.id), vec![]);
            for i in start_layer + 1..end_layer {
                dummy_nodes
                    .get_mut(&(from_node.id, to_node.id))
                    .unwrap()
                    .push((cur_pool.clone(), end_lane.clone(), Some(i)));
            }
        }
    }

    for ((from_id, to_id), dummy_list) in dummy_nodes {
        if dummy_list.is_empty() {
            continue;
        }

        let first_edge_id = graph.edges.len();
        let mut prev_id = from_id;
        for (pool, lane, layer) in dummy_list {
            let new_dummy_id = graph.add_dummy_node(pool, lane, layer);
            graph.add_dummy_edge(prev_id, new_dummy_id);
            prev_id = new_dummy_id;
        }
        graph.add_dummy_edge(prev_id, to_id);

        let old_edge_id = graph.remove_edge(from_id, to_id);
        let outgoing_index = graph.nodes[from_id.0]
            .outgoing
            .iter()
            .position(|edge_id| *edge_id == old_edge_id)
            .unwrap();
        graph.nodes[from_id.0].outgoing.remove(outgoing_index);
        let incoming_index = graph.nodes[to_id.0]
            .incoming
            .iter()
            .position(|edge_id| *edge_id == old_edge_id)
            .unwrap();
        graph.nodes[to_id.0].incoming.remove(incoming_index);
        graph.nodes[from_id.0].outgoing.push(EdgeId(first_edge_id));
        graph.nodes[to_id.0]
            .incoming
            .push(EdgeId(graph.edges.len() - 1));
    }
}
