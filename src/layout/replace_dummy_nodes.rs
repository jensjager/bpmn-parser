use crate::common::bpmn_event::BpmnEvent;
use crate::common::edge::Edge;
use crate::common::graph::{DataEdgeId, DataNodeId, EdgeId, Graph, NodeId};
use crate::common::node::Node;

pub fn replace_dummy_nodes(graph: &mut Graph) {
    replace_node_dummys(graph);
    replace_data_node_dummys(graph);
}

fn replace_data_node_dummys(graph: &mut Graph) {
    let mut new_data_edges: Vec<(DataNodeId, NodeId, bool, Vec<(f64, f64)>)> = Vec::new();
    for start_node in graph.data_nodes.iter() {
        let outgoing_data_edge_ids: Vec<&DataEdgeId> = start_node
            .outgoing
            .iter()
            .filter_map(|data_edge_id| {
                if graph.data_edges[data_edge_id.0].is_dummy {
                    Some(data_edge_id)
                } else {
                    None
                }
            })
            .collect();

        // TODO replace incoming dummy data edges
        let incoming_data_edge_ids: Vec<&DataEdgeId> = start_node
            .incoming
            .iter()
            .filter_map(|data_edge_id| {
                if graph.data_edges[data_edge_id.0].is_dummy {
                    Some(data_edge_id)
                } else {
                    None
                }
            })
            .collect();

        for edge_id in outgoing_data_edge_ids {
            let mut bend_points: Vec<(f64, f64)> = Vec::new();
            let edge = &graph.data_edges[edge_id.0];
            let to_node = &graph.nodes[edge.to.0];
            add_bend_points(
                &graph.nodes,
                &graph.edges,
                &to_node.outgoing[0],
                &mut bend_points,
            );
            let last_node_id = edge.text.as_ref().unwrap().parse::<usize>().unwrap();
            new_data_edges.push((start_node.id, NodeId(last_node_id), false, bend_points));
        }

        for edge_id in incoming_data_edge_ids {
            let mut bend_points: Vec<(f64, f64)> = Vec::new();
            let edge = &graph.data_edges[edge_id.0];
            let to_node = &graph.nodes[edge.to.0];
            add_bend_points(
                &graph.nodes,
                &graph.edges,
                &to_node.incoming[0],
                &mut bend_points,
            );
            let last_node_id = edge.text.as_ref().unwrap().parse::<usize>().unwrap();
            new_data_edges.push((start_node.id, NodeId(last_node_id), false, bend_points));
        }
    }

    new_data_edges
        .iter()
        .for_each(|(from, to, is_reversed, bend_points)| {
            if let Some(data_edge) = graph.data_edges.iter_mut().find(|data_edge| {
                data_edge.from == *from
                    && data_edge.to == *to
                    && data_edge.is_reversed == *is_reversed
            }) {
                data_edge.bend_points = Some(bend_points.clone());
                data_edge.temp_disabled = false;
            }
        });
}

fn replace_node_dummys(graph: &mut Graph) {
    let mut new_edges: Vec<(NodeId, NodeId, Vec<(f64, f64)>)> = Vec::new();
    for start_node in graph.nodes.iter() {
        if start_node.event == Some(BpmnEvent::Dummy()) {
            continue;
        }
        let outgoing_edge_ids: Vec<&EdgeId> = start_node
            .outgoing
            .iter()
            .filter_map(|edge_id| {
                if graph.edges[edge_id.0].is_dummy {
                    Some(edge_id)
                } else {
                    None
                }
            })
            .collect();

        for edge_id in outgoing_edge_ids {
            let mut bend_points: Vec<(f64, f64)> = Vec::new();
            let last_edge = add_bend_points(&graph.nodes, &graph.edges, edge_id, &mut bend_points);
            new_edges.push((start_node.id, last_edge.to, bend_points));
        }
    }
    new_edges.iter().for_each(|(from, to, bend_points)| {
        if let Some(edge) = graph
            .edges
            .iter_mut()
            .find(|edge| edge.from == *from && edge.to == *to)
        {
            edge.bend_points = Some(bend_points.clone());
            edge.temp_disabled = false;
        }
    });

    // graph.nodes.iter_mut().for_each(|node| {
    //     let mut to_be_removed_out = Vec::new();
    //     let mut to_be_removed_in = Vec::new();
    //     for (i, outgoing) in node.outgoing.iter().enumerate() {
    //         if graph.edges[outgoing.0].is_dummy {
    //             to_be_removed_out.push(i);
    //         }
    //     }
    //     for (i, incoming) in node.incoming.iter().enumerate() {
    //         if graph.edges[incoming.0].is_dummy {
    //             to_be_removed_in.push(i);
    //         }
    //     }
    //     for (i, x) in to_be_removed_out.iter().enumerate() {
    //         if i == 0 {
    //             node.outgoing.remove(*x);
    //         } else {
    //             node.outgoing.remove(*x - i);
    //         }
    //     }
    //     for (i, x) in to_be_removed_in.iter().enumerate() {
    //         if i == 0 {
    //             node.incoming.remove(*x);
    //         } else {
    //             node.incoming.remove(*x - i);
    //         }
    //     }
    // });
}

fn add_bend_points(
    nodes: &Vec<Node>,
    edges: &Vec<Edge>,
    edge_id: &EdgeId,
    bend_points: &mut Vec<(f64, f64)>,
) -> Edge {
    let edge = &edges[edge_id.0];
    let to_node = &nodes[edge.to.0];
    edge.bend_points
        .as_ref()
        .unwrap()
        .iter()
        .for_each(|bend_point| bend_points.push(*bend_point));
    if to_node.event == Some(BpmnEvent::Dummy()) {
        if to_node.outgoing.is_empty() {
            return edge.clone();
        } else {
            return add_bend_points(nodes, edges, &to_node.outgoing[0], bend_points);
        }
    }

    edge.clone()
}

fn add_bend_points_incoming(
    nodes: &Vec<Node>,
    edges: &Vec<Edge>,
    edge_id: &EdgeId,
    bend_points: &mut Vec<(f64, f64)>,
) -> Edge {
    let edge = &edges[edge_id.0];
    let from_node = &nodes[edge.from.0];
    edge.bend_points
        .as_ref()
        .unwrap()
        .iter()
        .for_each(|bend_point| bend_points.push(*bend_point));
    if from_node.event == Some(BpmnEvent::Dummy()) {
        if from_node.incoming.is_empty() {
            return edge.clone();
        } else {
            return add_bend_points_incoming(nodes, edges, &from_node.incoming[0], bend_points);
        }
    }

    edge.clone()
}
