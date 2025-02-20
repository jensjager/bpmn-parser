use crate::common::node::Node;

use crate::common::graph::Graph;

pub fn assign_xy_to_data_nodes(graph: &mut Graph) {
    for data_node in graph.data_nodes.iter_mut() {
        let mut x_pos = get_x_pos(data_node.layer_id.unwrap());
        if data_node.uses_half_layer {
            x_pos += 80.0;
        }
        let y_pos = get_y_pos(
            data_node.layer_id.unwrap(),
            data_node.lane.as_ref().unwrap(),
            &graph.nodes,
        );
        let x_offset = 20.0;
        let y_offset = 0.0;

        data_node.set_position(x_pos, y_pos - 50.0, x_offset, y_offset);
    }
}

fn get_x_pos(layer: usize) -> f64 {
    layer as f64 * 150.0 + 180.0
}

fn get_y_pos(layer: usize, lane: &String, nodes: &[Node]) -> f64 {
    nodes
        .iter()
        .filter_map(|node| {
            if node.lane.as_ref().unwrap() == lane && node.layer_id.unwrap() == layer {
                node.y
            } else {
                None
            }
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}
