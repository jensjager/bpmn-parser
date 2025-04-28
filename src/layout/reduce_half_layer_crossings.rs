use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::layout::xy_ilp::HALF_LAYER_OFFSET;

pub fn reduce_half_layer_crossings(graph: &mut Graph) {
    for data_node in graph.data_nodes.iter_mut() {
        if !data_node.uses_half_layer {
            continue;
        }

        let lane = graph
            .pools
            .iter_mut()
            .find(|pool| pool.pool_name == data_node.pool)
            .unwrap()
            .lanes
            .iter_mut()
            .find(|lane| lane.lane == data_node.lane)
            .unwrap();

        let mut nodes_in_dn_layer: Vec<&Node> = Vec::new();
        lane.nodes.iter().for_each(|nodeid| {
            if graph.nodes[nodeid.0].layer_id == data_node.layer_id {
                nodes_in_dn_layer.push(&graph.nodes[nodeid.0]);
            }
        });

        let mut crossing = false;
        for node in &nodes_in_dn_layer {
            if crossing {
                break;
            }
            for edgeid in &node.outgoing {
                let edge = &graph.edges[edgeid.0];
                let to_node = &graph.nodes[edge.to.0];
                if node.y < to_node.y {
                    if node.y < data_node.y && data_node.y < to_node.y {
                        crossing = true;
                        break;
                    }
                } else {
                    if node.y > data_node.y && to_node.y < data_node.y {
                        crossing = true;
                        break;
                    }
                }
            }
        }

        if crossing {
            dbg!(&data_node);
            data_node.uses_half_layer = false;
            data_node.x_offset = Some(data_node.x_offset.unwrap() - HALF_LAYER_OFFSET);
        }
    }
}
