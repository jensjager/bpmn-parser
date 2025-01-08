use crate::common::graph::NodeId;
use crate::common::{bpmn_event::get_node_size, graph::Graph, lane::Lane, node::Node};
use std::collections::HashMap;

pub fn assign_xy_to_nodes(graph: &mut Graph) {
    let pool_position_x = 100.0;
    let mut pool_position_y = 100.0;
    let mut node_position_y = 150.0;
    let layer_width = 150.0;
    let lane_x_offset = 30.0;
    let lane_position_x = pool_position_x + lane_x_offset;
    let mut lane_position_y = 100.0;
    let node_x_start = lane_position_x + 50.0;

    let mut original_positions: HashMap<NodeId, f64> = HashMap::new();

    {
        let pools = &mut graph.pools;
        for pool in pools {
            let mut pool_height = 0.0;
            let mut lane_width = 0.0;
            for lane in &mut pool.lanes {
                lane.sort_nodes_by_layer_id(&graph.nodes);
                let max_height = find_max_nodes_in_layer(&lane.lane, &graph.nodes) * 100 + 80;
                pool_height += max_height as f64;
                lane.height = Some(max_height as f64);
                let new_lane_width = get_lane_width(lane, &graph.nodes);
                if new_lane_width > lane_width {
                    lane_width = new_lane_width;
                }

                for layer_index in 0..lane.nodes.len() {
                    let x = node_x_start + (layer_index as f64 * layer_width);
                    let mut y_layer_position = node_position_y;
                    {
                        let nodes_for_this_layer =
                            lane.get_nodes_by_layer_id(layer_index, &mut graph.nodes);
                        for node in nodes_for_this_layer {
                            let (node_size_x, node_size_y) =
                                get_node_size(node.event.as_ref().unwrap());
                            let y_offset = if node_size_y < 80 {
                                (80 - node_size_y) as f64 / 2.0
                            } else {
                                0.0
                            };
                            let x_offset = if node_size_x < 100 {
                                (100 - node_size_x) as f64 / 2.0
                            } else {
                                0.0
                            };
                            let old_y = node.y.unwrap_or(y_layer_position);
                            node.set_position(x, old_y, x_offset, y_offset);
                            original_positions.insert(node.id, x);
                            y_layer_position += 100.0;
                        }
                    }
                }

                node_position_y += max_height as f64;
                lane.x = Some(lane_position_x);
                lane.y = Some(lane_position_y);
                lane_position_y += max_height as f64;
            }

            if lane_width > pool.width.unwrap_or(0.0) {
                pool.width = Some(lane_width + lane_x_offset);
            }
            pool.height = Some(pool_height);
            pool.x = Some(pool_position_x);
            pool.y = Some(pool_position_y);
            pool_position_y += pool_height;
            pool.set_lane_width(lane_width);
        }

        let mut lane_change_new_x: HashMap<NodeId, f64> = HashMap::new();
        {
            let edges = &graph.edges;
            for edge in edges {
                let to_node = &graph.nodes[edge.to.0];
                let from_node = &graph.nodes[edge.from.0];
                if from_node.lane != to_node.lane {
                    if let Some(fx) = from_node.x {
                        lane_change_new_x.insert(to_node.id, fx);
                    }
                }
            }
        }

        let mut lane_shifts: HashMap<(Option<String>, Option<String>), Vec<(usize, NodeId, f64)>> =
            HashMap::new();

        {
            for pool in &graph.pools {
                for lane in &pool.lanes {
                    let layer_nodes = &lane.nodes;
                    for (index, node_id) in layer_nodes.iter().enumerate() {
                        if let Some(&new_x) = lane_change_new_x.get(&node_id) {
                            if let Some(&old_x) = original_positions.get(&node_id) {
                                let dx = new_x - old_x;
                                if dx.abs() > f64::EPSILON {
                                    lane_shifts
                                        .entry((pool.pool_name.clone(), lane.lane.clone()))
                                        .or_default()
                                        .push((index, *node_id, dx));
                                }
                            }
                        }
                    }
                }
            }
        }

        for changes_key in lane_shifts.keys() {
            let mut changes = lane_shifts.get(changes_key).unwrap().clone();
            changes.sort_by_key(|c| c.0);
        }

        {
            for pool in &mut graph.pools {
                for lane in &mut pool.lanes {
                    let key = (pool.pool_name.clone(), lane.lane.clone());
                    if let Some(mut changes) = lane_shifts.get(&key).cloned() {
                        changes.sort_by_key(|c| c.0);

                        let len = lane.nodes.len();
                        let mut dx_map: Vec<f64> = vec![0.0; len];

                        for (node_index, _node_id, dx) in changes {
                            for i in node_index..len {
                                dx_map[i] += dx;
                            }
                        }

                        for i in 1..len {
                            dx_map[i] += dx_map[i - 1];
                        }

                        for (node_id, dx) in lane.nodes.iter().zip(dx_map) {
                            if dx.abs() > f64::EPSILON {
                                let node = &mut graph.nodes[node_id.0];
                                let old_x = node.x.unwrap_or(0.0);
                                let old_y = node.y.unwrap_or(0.0);
                                let old_y_off = node.y_offset.unwrap_or(0.0);
                                let old_x_off = node.x_offset.unwrap_or(0.0);
                                node.set_position(old_x + dx, old_y, old_x_off, old_y_off);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn find_max_nodes_in_layer(lane: &Option<String>, nodes: &[Node]) -> usize {
    let mut max = 0;
    let mut cur_max = 0;
    let mut current_layer_id = 0;

    for node in nodes {
        if node.lane != *lane {
            continue;
        }
        if node.layer_id.unwrap_or(0) != current_layer_id {
            current_layer_id = node.layer_id.unwrap_or(0);
            if cur_max > max {
                max = cur_max;
            }

            cur_max = 1;
        } else if node.layer_id.unwrap_or(0) == current_layer_id {
            cur_max += 1;
        }
    }
    max
}

fn get_lane_width(lane: &Lane, nodes: &[Node]) -> f64 {
    let last_node = &nodes[lane.nodes.last().unwrap().0];
    let last_layer = last_node.layer_id.unwrap_or(0);
    println!("last_layer: {}", last_layer);
    if last_layer == 0 || last_layer == 1 {
        return 350.0;
    } else {
        return (last_layer) as f64 * 300.0;
    }
}
