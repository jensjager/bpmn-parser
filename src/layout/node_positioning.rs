use crate::common::{bpmn_event::*, graph::Graph, node::Node, lane::Lane};

pub fn assign_xy_to_nodes(graph: &mut Graph) {
    let x_start = 150.0; // Initial x position
    let mut y_position = 150.0; // Initial y position
    let layer_width = 150.0; // Fixed width between layers
    

    for pool in graph.get_pools_mut() {
        let mut pool_height = 0.0;
        let mut pool_width = 0.0;
        for lane in pool.get_lanes_mut() {
            lane.sort_nodes_by_layer_id();
            let max_height = find_max_nodes_in_layer(lane.get_layers()) * 100 + 100;
            pool_height += max_height as f64;
            let lane_width = get_lane_width(lane);
            if lane_width > pool_width {
                pool_width = lane_width;
            }

            lane.set_width(pool_width);
            lane.set_height(max_height as f64);

            for layer_index in 0..lane.get_layers().len() {
                let x = x_start + (layer_index as f64 * layer_width); // X position based on layer index
                let mut y_layer_position = y_position;
                
                for node in lane.get_nodes_by_layer_id_mut(layer_index) {
                    let (_, node_size_y) = get_node_size(&node.event.as_ref().unwrap());
                    let y_offset = if node_size_y < 80 {
                        (80 - node_size_y) as f64 / 2.0
                    } else {
                        0.0
                    };

                    node.set_position(x, y_layer_position, y_offset);
                    y_layer_position += node_size_y as f64 + 100.0; // Move y position in layer
                }
            }

            y_position += max_height as f64; // Move global y position
        }

        pool.set_width(pool_width);
        pool.set_height(pool_height);
    }
}

fn find_max_nodes_in_layer(nodes: &Vec<Node>) -> usize {
    let mut max = 0;
    let mut current_layer_id = 0;

    for node in nodes {
        if node.layer_id.unwrap_or(0) != current_layer_id {
            current_layer_id = node.layer_id.unwrap_or(0);
            max = 0;
        }
        if node.layer_id.unwrap_or(0) == current_layer_id {
            max += 1;
        }
    }

    max
}

fn get_lane_width(lane: &Lane) -> f64 {
    let last_node = lane.get_layers().last().unwrap();
    let last_layer = last_node.layer_id.unwrap_or(0);
    println!("last_layer: {}", last_layer);
    if last_layer == 0 || last_layer == 1 {
        return 200.0;
    } else {
       return (last_layer) as f64 * 200.0;
    }
}
