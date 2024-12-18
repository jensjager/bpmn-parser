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

    let mut original_positions: HashMap<usize, f64> = HashMap::new();

    {
        let pools = graph.get_pools_mut();
        for pool in pools {
            let mut pool_height = 0.0;
            let mut lane_width = 0.0;
            for lane in pool.get_lanes_mut() {
                lane.sort_nodes_by_layer_id();
                let max_height = find_max_nodes_in_layer(lane.get_layers()) * 100 + 80;
                pool_height += max_height as f64;
                lane.set_height(max_height as f64);
                let new_lane_width = get_lane_width(lane);
                if new_lane_width > lane_width {
                    lane_width = new_lane_width;
                }

                for layer_index in 0..lane.get_layers().len() {
                    let x = node_x_start + (layer_index as f64 * layer_width);
                    let mut y_layer_position = node_position_y;
                    {
                        let nodes_for_this_layer = lane.get_nodes_by_layer_id_mut(layer_index);
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
                lane.set_position(lane_position_x, lane_position_y);
                lane_position_y += max_height as f64;
            }

            if lane_width > pool.width.unwrap_or(0.0) {
                pool.set_width(lane_width + lane_x_offset);
            }
            pool.set_height(pool_height);
            pool.set_position(pool_position_x, pool_position_y);
            pool_position_y += pool_height;
            pool.set_lane_width(lane_width);
        }

        //     let mut lane_change_new_x: HashMap<usize, f64> = HashMap::new();
        //     {
        //         let edges = &graph.edges;
        //         for edge in edges {
        //             if let (Some(to_node), Some(from_node)) = (
        //                 graph.get_node_by_id(edge.to),
        //                 graph.get_node_by_id(edge.from),
        //             ) {
        //                 if from_node.lane != to_node.lane {
        //                     if let Some(fx) = from_node.x {
        //                         lane_change_new_x.insert(to_node.id, fx);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     let mut lane_shifts: HashMap<(String, String), Vec<(usize, usize, f64)>> = HashMap::new();

        //     {
        //         let pools = graph.get_pools();
        //         for pool in pools {
        //             for lane in pool.get_lanes() {
        //                 let layer_nodes = lane.get_layers();
        //                 for (index, node) in layer_nodes.iter().enumerate() {
        //                     if let Some(&new_x) = lane_change_new_x.get(&node.id) {
        //                         if let Some(&old_x) = original_positions.get(&node.id) {
        //                             let dx = new_x - old_x;
        //                             if dx.abs() > f64::EPSILON {
        //                                 let key = (pool.get_pool_name(), lane.get_lane().clone());
        //                                 lane_shifts
        //                                     .entry(key)
        //                                     .or_default()
        //                                     .push((index, node.id, dx));
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     for changes_key in lane_shifts.keys() {
        //         let mut changes = lane_shifts.get(changes_key).unwrap().clone();
        //         changes.sort_by_key(|c| c.0);
        //     }

        //     {
        //         let pools = graph.get_pools_mut();
        //         for pool in pools {
        //             let pool_name = pool.get_pool_name().to_string(); // Salvestame pooli nime ette
        //             for lane in pool.get_lanes_mut() {
        //                 let lane_name = lane.get_lane().clone(); // Salvestame lane'i nime ette
        //                 let key = (pool_name.clone(), lane_name);
        //                 if let Some(mut changes) = lane_shifts.get(&key).cloned() {
        //                     changes.sort_by_key(|c| c.0);

        //                     let layer_nodes = lane.get_layers();
        //                     let len = layer_nodes.len();
        //                     let mut dx_map: Vec<f64> = vec![0.0; len];

        //                     for (node_index, _node_id, dx) in changes {
        //                         for i in node_index..len {
        //                             dx_map[i] += dx;
        //                         }
        //                     }

        //                     for i in 1..len {
        //                         dx_map[i] += dx_map[i - 1];
        //                     }

        //                     let lane_nodes_mut = lane.get_layers_mut();
        //                     for i in 0..len {
        //                         if dx_map[i].abs() > f64::EPSILON {
        //                             let node = &mut lane_nodes_mut[i];
        //                             let old_x = node.x.unwrap_or(0.0);
        //                             let old_y = node.y.unwrap_or(0.0);
        //                             let old_y_off = node.y_offset.unwrap_or(0.0);
        //                             let old_x_off = node.x_offset.unwrap_or(0.0);
        //                             node.set_position(old_x + dx_map[i], old_y, old_x_off, old_y_off);
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
    }
}

fn find_max_nodes_in_layer(nodes: &Vec<Node>) -> usize {
    let mut max = 0;
    let mut cur_max = 0;
    let mut current_layer_id = 0;

    for node in nodes {
        println!("Node max: {}", node.layer_id.as_ref().unwrap());
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
    println!("Max nodes found: {}", max);
    max
}

fn get_lane_width(lane: &Lane) -> f64 {
    let last_node = lane.get_layers().last().unwrap();
    let last_layer = last_node.layer_id.unwrap_or(0);
    println!("last_layer: {}", last_layer);
    if last_layer == 0 || last_layer == 1 {
        return 350.0;
    } else {
        return (last_layer) as f64 * 200.0;
    }
}
