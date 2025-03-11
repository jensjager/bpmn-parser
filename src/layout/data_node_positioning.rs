use crate::common::datanode::DataNode;
use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::common::pool::Pool;

const DN_SPACE: f64 = 60.0;

pub fn assign_xy_to_data_nodes(graph: &mut Graph) {
    let x_offset = 20.0;
    let y_offset = 0.0;
    for i in 0..graph.data_nodes.len() {
        let (prev_nodes, rest) = graph.data_nodes.split_at_mut(i);
        let data_node = &mut rest[0];
        let (x_pos, y_pos) = get_x_y(&graph.nodes, data_node);
        // First datanode
        if i == 0 {
            push_graph(
                &mut graph.pools,
                &mut graph.nodes,
                data_node,
                data_node.above,
            );
            data_node.set_position(x_pos, y_pos - 20.0, x_offset, y_offset);
        } else {
            let prev_node = &prev_nodes[i - 1];
            let prev_node_lane = prev_node.lane.clone();
            let prev_node_layer_id = prev_node.layer_id.clone();
            let prev_node_uses_half_layer = prev_node.uses_half_layer.clone();
            let prev_x = prev_node.x.clone();
            let prev_y = prev_node.y.clone();

            // Is in different lane
            if data_node.lane != prev_node_lane {
                push_graph(
                    &mut graph.pools,
                    &mut graph.nodes,
                    data_node,
                    data_node.above,
                );
                data_node.set_position(x_pos, y_pos, x_offset, y_offset);
            } else {
                // Is in the same layer
                if data_node.layer_id == prev_node_layer_id
                    && data_node.uses_half_layer == prev_node_uses_half_layer
                {
                    let new_x = prev_x.unwrap();
                    let new_y = prev_y.unwrap() + DN_SPACE;
                    push_graph(
                        &mut graph.pools,
                        &mut graph.nodes,
                        data_node,
                        data_node.above,
                    );
                    data_node.set_position(x_pos, new_y, x_offset, y_offset);
                // Is in a different layer
                } else {
                    data_node.set_position(x_pos, y_pos, x_offset, y_offset);
                }
            }
        }
    }
}

fn push_graph(pools: &mut Vec<Pool>, nodes: &mut Vec<Node>, data_node: &mut DataNode, above: bool) {
    let mut past_lane = false;
    for pool in pools.iter_mut() {
        for lane in pool.lanes.iter_mut() {
            let in_lane = lane.lane == data_node.lane;
            if in_lane && !past_lane {
                if !above {
                    lane.height = Some(lane.height.unwrap() + DN_SPACE);
                    past_lane = true;
                }

                continue;
            }

            if past_lane {
                for nodeid in lane.nodes.iter_mut() {
                    let node = &mut nodes[nodeid.0];
                    if in_lane {
                        node.y = Some(node.y.unwrap() + DN_SPACE);
                    }
                }
                if in_lane {
                    lane.height = Some(lane.height.unwrap() + DN_SPACE);
                } else {
                    lane.y = Some(lane.y.unwrap() + DN_SPACE);
                }
            }
        }
        if past_lane {
            if pool.pool_name != data_node.pool {
                pool.y = Some(pool.y.unwrap() + DN_SPACE);
            } else {
                pool.height = Some(pool.height.unwrap() + DN_SPACE);
            }
        }
    }
}

fn get_x_y(nodes: &Vec<Node>, data_node: &mut DataNode) -> (f64, f64) {
    let mut x_pos = get_x_pos(data_node.layer_id.unwrap());
    if data_node.uses_half_layer {
        x_pos += 80.0;
    }

    let y_pos = get_y_pos(
        data_node.layer_id.unwrap(),
        &data_node.lane,
        nodes,
        data_node.above,
    );

    (x_pos, y_pos)
}

fn get_x_pos(layer: usize) -> f64 {
    layer as f64 * 150.0 + 180.0
}

fn get_y_pos(layer: usize, lane: &Option<String>, nodes: &[Node], above: bool) -> f64 {
    if above {
        nodes
            .iter()
            .filter_map(|node| {
                if node.lane == *lane && node.layer_id.unwrap() == layer {
                    node.y
                } else {
                    None
                }
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or_else(|| {
                nodes
                    .iter()
                    .enumerate()
                    .filter_map(|(index, node)| {
                        if node.lane == *lane {
                            if index != 0
                                && nodes
                                    .get(index + 1)
                                    .map_or(true, |next_node| next_node.lane != *lane)
                            {
                                return node.y;
                            }
                        }
                        None
                    })
                    .next()
                    .unwrap_or(0.0)
            })
    } else {
        nodes
            .iter()
            .filter_map(|node| {
                if node.lane == *lane && node.layer_id.unwrap() == layer {
                    node.y
                } else {
                    None
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}
