use crate::common::{bpmn_event::*, layer::Layer, pool::Pool};

pub fn assign_xy_to_nodes(pools: &mut Vec<Pool>) {
    let x_start = 100.0; // Initial x position
    let mut y_position = 100.0; // Initial y position
    let layer_width = 150.0; // Fixed width between layers

    for pool in pools {
        for lane in pool.get_lanes_mut() {
            let max_height = find_max_nodes_in_layer(lane.get_layers()) * 100;

            for (layer_index, layer) in lane.get_layers_mut().iter_mut().enumerate() {
                let x = x_start + (layer_index as f64 * layer_width); // X position based on layer index
                let mut y_layer_position = y_position;

                for node in layer.get_nodes_mut() {
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
    }
}

fn find_max_nodes_in_layer(layers: &Vec<Layer>) -> usize {
    let mut max = 0;

    for layer in layers {
        let nodes = layer.get_nodes();
        if nodes.len() > max {
            max = nodes.len();
        }
    }

    max
}
