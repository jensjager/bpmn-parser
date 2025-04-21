use crate::common::bpmn_event::BpmnEvent;
use crate::common::edge::Edge;
use crate::common::graph::{DataNodeId, EdgeId, Graph, NodeId};
use crate::common::node::Node;

pub fn replace_dummy_nodes(graph: &mut Graph) {
    replace_node_dummys(graph);
}

fn replace_node_dummys(graph: &mut Graph) {
    let mut new_edges: Vec<(NodeId, NodeId, bool, bool, Vec<(f64, f64)>)> = Vec::new();
    for start_node in graph.nodes.iter() {
        if start_node.event == Some(BpmnEvent::Dummy()) {
            continue;
        }
        let outgoing_edge_ids: Vec<&EdgeId> = start_node
            .outgoing
            .iter()
            .filter_map(|edge_id| {
                if let Some(dummy) = graph.edges[edge_id.0].dummy.as_ref() {
                    if dummy.skip {
                        None
                    } else {
                        Some(edge_id)
                    }
                } else {
                    None
                }
            })
            .collect();

        for edge_id in outgoing_edge_ids {
            let mut bend_points: Vec<(f64, f64)> = Vec::new();
            let (last_edge, is_data) =
                add_bend_points(&graph.nodes, &graph.edges, edge_id, &mut bend_points);
            if is_data {
                let last_edge = &graph
                    .data_edges
                    .iter()
                    .find(|data_edge| {
                        data_edge.from
                            == DataNodeId(last_edge.dummy.as_ref().unwrap().from.unwrap())
                            && data_edge.to == last_edge.to
                            && data_edge.is_reversed
                    })
                    .unwrap();
                let last_edge = &graph
                    .data_edges
                    .iter()
                    .find(|data_edge| data_edge.to == last_edge.to && data_edge.is_reversed)
                    .unwrap();
                last_edge
                    .bend_points
                    .as_ref()
                    .unwrap()
                    .iter()
                    .for_each(|bend_point| bend_points.push(*bend_point));
                new_edges.push((
                    start_node.id,
                    NodeId(last_edge.from.0),
                    true,
                    true,
                    bend_points,
                ));
            } else {
                new_edges.push((start_node.id, last_edge.to, false, false, bend_points));
            }
        }

        let incoming_edge_ids: Vec<&EdgeId> = start_node
            .incoming
            .iter()
            .filter_map(|edge_id| {
                if let Some(dummy) = graph.edges[edge_id.0].dummy.as_ref() {
                    if dummy.skip {
                        None
                    } else {
                        Some(edge_id)
                    }
                } else {
                    None
                }
            })
            .collect();

        for edge_id in incoming_edge_ids {
            let mut bend_points: Vec<(f64, f64)> = Vec::new();
            let (last_edge, is_data) =
                add_bend_points_incoming(&graph.nodes, &graph.edges, edge_id, &mut bend_points);
            if is_data {
                let last_edge = &graph
                    .data_edges
                    .iter()
                    .find(|data_edge| data_edge.to == last_edge.from && !data_edge.is_reversed)
                    .unwrap();
                last_edge
                    .bend_points
                    .as_ref()
                    .unwrap()
                    .iter()
                    .rev()
                    .for_each(|bend_point| bend_points.push(*bend_point));

                new_edges.push((
                    start_node.id,
                    NodeId(last_edge.from.0),
                    true,
                    false,
                    bend_points,
                ));
            } else {
                new_edges.push((start_node.id, last_edge.to, false, false, bend_points));
            }
        }
    }
    new_edges
        .iter_mut()
        .for_each(|(from, to, is_data, is_reversed, bend_points)| {
            if !*is_data {
                if let Some(edge) = graph
                    .edges
                    .iter_mut()
                    .find(|edge| edge.from == *from && edge.to == *to)
                {
                    edge.bend_points = Some(bend_points.clone());
                    edge.temp_disabled = false;
                }
            } else {
                if let Some(data_edge) = graph.data_edges.iter_mut().find(|data_edge| {
                    data_edge.from.0 == to.0
                        && data_edge.to == *from
                        && data_edge.is_reversed == *is_reversed
                }) {
                    if *is_reversed {
                        bend_points.reverse();
                    }
                    data_edge.bend_points = Some(bend_points.clone());
                    data_edge.temp_disabled = false;
                }
            }
        });
}

fn add_bend_points(
    nodes: &Vec<Node>,
    edges: &Vec<Edge>,
    edge_id: &EdgeId,
    bend_points: &mut Vec<(f64, f64)>,
) -> (Edge, bool) {
    let edge = &edges[edge_id.0];
    let to_node = &nodes[edge.to.0];
    edge.bend_points
        .as_ref()
        .unwrap()
        .iter()
        .for_each(|bend_point| bend_points.push(*bend_point));
    if to_node.event == Some(BpmnEvent::Dummy()) {
        if to_node.outgoing.is_empty() {
            return (edge.clone(), true);
        } else {
            return add_bend_points(nodes, edges, &to_node.outgoing[0], bend_points);
        }
    }

    (edge.clone(), false)
}

fn add_bend_points_incoming(
    nodes: &Vec<Node>,
    edges: &Vec<Edge>,
    edge_id: &EdgeId,
    bend_points: &mut Vec<(f64, f64)>,
) -> (Edge, bool) {
    let edge = &edges[edge_id.0];
    let from_node = &nodes[edge.from.0];
    edge.bend_points
        .as_ref()
        .unwrap()
        .iter()
        .rev()
        .for_each(|bend_point| bend_points.push(*bend_point));
    if from_node.event == Some(BpmnEvent::Dummy()) {
        if from_node.incoming.is_empty() {
            return (edge.clone(), true);
        } else {
            return add_bend_points_incoming(nodes, edges, &from_node.incoming[0], bend_points);
        }
    }

    (edge.clone(), false)
}
