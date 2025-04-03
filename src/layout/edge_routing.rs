use std::collections::HashMap;

use crate::common::bpmn_event::get_node_size;
use crate::common::datanode::DataNode;
use crate::common::graph::{EdgeId, Graph};
use crate::common::node::Node;
use crate::layout;

#[derive(Debug, Copy, Clone)]
struct RoutingEdge {
    id: EdgeId,
    from: usize,
    to: usize,
    is_reversed: Option<bool>,
    direction: Direction,
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Right,
}

pub fn edge_routing(graph: &mut Graph) {
    // Hashmap<(Lane, LayerId), Vec<RoutingEdge>>
    let mut layered_edges = get_layered_edges(graph);

    let mut sorted_edges: HashMap<Option<usize>, HashMap<usize, Vec<RoutingEdge>>> = HashMap::new();
    for ((lane, layer_id), routing_edges) in layered_edges.iter_mut() {
        sorted_edges.insert(*layer_id, route_edges(routing_edges));
    }

    add_bend_points(graph, sorted_edges);
}

fn route_edges(routing_edges: &mut Vec<RoutingEdge>) -> HashMap<usize, Vec<RoutingEdge>> {
    let mut sorted_edges: HashMap<usize, Vec<RoutingEdge>> = HashMap::new();

    for routing_edge in routing_edges.iter_mut() {
        let mut inserted = false;
        for (i, edges) in sorted_edges.iter_mut() {
            if edges
                .iter()
                .find(|edge| edge.from == routing_edge.from && routing_edge.is_reversed.is_none())
                .is_some()
            {
                edges.push(*routing_edge);
                inserted = true;
                break;
            }
        }
        if !inserted {
            if sorted_edges.is_empty() {
                sorted_edges.insert(0, vec![*routing_edge]);
            } else {
                sorted_edges.insert(sorted_edges.len(), vec![*routing_edge]);
            }
        }
    }
    sorted_edges
}

fn should_come_before(a: &RoutingEdge, b: &RoutingEdge) -> bool {
    let in_between = a.from >= b.from && a.to <= b.to;

    let covers = a.from < b.to && a.to > b.to;

    in_between || covers
}

fn get_layered_edges(
    graph: &mut Graph,
) -> HashMap<(Option<String>, Option<usize>), Vec<RoutingEdge>> {
    let mut layered_edges: HashMap<(Option<String>, Option<usize>), Vec<RoutingEdge>> =
        HashMap::new();
    for (i, edge) in graph.edges.iter().enumerate() {
        if edge.temp_disabled {
            continue;
        }
        let from_node = &graph.nodes[edge.from.0];
        let to_node = &graph.nodes[edge.to.0];
        layered_edges
            .entry((from_node.lane.clone(), from_node.layer_id.clone()))
            .or_insert(vec![])
            .push(RoutingEdge {
                id: EdgeId(i),
                from: edge.from.0,
                to: edge.to.0,
                is_reversed: None,
                direction: if edge.from.0 < edge.to.0 {
                    Direction::Down
                } else if edge.from.0 > edge.to.0 {
                    Direction::Up
                } else {
                    Direction::Right
                },
            });
    }
    for (i, data_edge) in graph.data_edges.iter().enumerate() {
        let from_node = &graph.data_nodes[data_edge.from.0];
        let to_node = &graph.nodes[data_edge.to.0];
        if data_edge.is_reversed {
            layered_edges
                .entry((to_node.lane.clone(), to_node.layer_id.clone()))
                .or_insert(vec![])
                .push(RoutingEdge {
                    id: EdgeId(i),
                    from: data_edge.to.0,
                    to: data_edge.from.0,
                    is_reversed: Some(true),
                    direction: if data_edge.to.0 < data_edge.from.0 {
                        Direction::Down
                    } else if data_edge.to.0 > data_edge.from.0 {
                        Direction::Up
                    } else {
                        Direction::Right
                    },
                });
        } else {
            layered_edges
                .entry((from_node.lane.clone(), from_node.layer_id.clone()))
                .or_insert(vec![])
                .push(RoutingEdge {
                    id: EdgeId(i),
                    from: data_edge.from.0,
                    to: data_edge.to.0,
                    is_reversed: Some(false),
                    direction: if data_edge.from.0 < data_edge.to.0 {
                        Direction::Down
                    } else if data_edge.from.0 > data_edge.to.0 {
                        Direction::Up
                    } else {
                        Direction::Right
                    },
                });
        }
    }
    layered_edges
}

fn add_bend_points(
    graph: &mut Graph,
    sorted_edges: HashMap<Option<usize>, HashMap<usize, Vec<RoutingEdge>>>,
) {
    for (_, layered_edges) in sorted_edges.iter() {
        let layer_size = layered_edges.len();
        for (i, routing_edges) in layered_edges.iter() {
            for routing_edge in routing_edges.iter() {
                let mut bend_points = vec![];
                if routing_edge.is_reversed.is_some() {
                    if routing_edge.is_reversed.unwrap() {
                        bend_points.push(get_right_port(&graph.nodes[routing_edge.from]));
                        get_data_mid_point_reversed(
                            &i,
                            &layer_size,
                            &graph.data_nodes[routing_edge.to],
                            &graph.nodes[routing_edge.from],
                        )
                        .iter()
                        .for_each(|bend_point| {
                            bend_points.push(*bend_point);
                        });
                        bend_points.push(get_data_left_port(&graph.data_nodes[routing_edge.to]));
                    } else {
                        bend_points.push(get_data_right_port(&graph.data_nodes[routing_edge.from]));
                        get_data_mid_point(
                            &i,
                            &layer_size,
                            &graph.data_nodes[routing_edge.from],
                            &graph.nodes[routing_edge.to],
                        )
                        .iter()
                        .for_each(|bend_point| {
                            bend_points.push(*bend_point);
                        });
                        bend_points.push(get_left_port(&graph.nodes[routing_edge.to]));
                    }
                } else {
                    bend_points.push(get_right_port(&graph.nodes[routing_edge.from]));
                    get_mid_point(
                        &i,
                        &layer_size,
                        &graph.nodes[routing_edge.from],
                        &graph.nodes[routing_edge.to],
                    )
                    .iter()
                    .for_each(|bend_point| {
                        bend_points.push(*bend_point);
                    });
                    bend_points.push(get_left_port(&graph.nodes[routing_edge.to]));
                }

                if routing_edge.is_reversed.is_some() {
                    graph.data_edges[routing_edge.id.0].bend_points = Some(bend_points);
                } else {
                    graph.edges[routing_edge.id.0].bend_points = Some(bend_points);
                }
            }
        }
    }
}

fn get_left_port(node: &Node) -> (f64, f64) {
    let (_, height) = get_node_size(&node.event.clone().unwrap());
    let (x, y, x_offset, y_offset) = (
        node.x.unwrap(),
        node.y.unwrap(),
        node.x_offset.unwrap(),
        node.y_offset.unwrap(),
    );
    (x + x_offset, y + height as f64 / 2.0 + y_offset)
}

fn get_right_port(node: &Node) -> (f64, f64) {
    let (width, height) = get_node_size(&node.event.clone().unwrap());
    let (x, y, x_offset, y_offset) = (
        node.x.unwrap(),
        node.y.unwrap(),
        node.x_offset.unwrap(),
        node.y_offset.unwrap(),
    );
    (
        x + width as f64 + x_offset,
        y + height as f64 / 2.0 + y_offset,
    )
}

fn get_mid_point(index: &usize, size: &usize, from_node: &Node, to_node: &Node) -> Vec<(f64, f64)> {
    let (from_x, from_y) = get_right_port(from_node);
    let (_, to_y) = get_right_port(to_node);
    if *size == 1 {
        let mid_x = from_x + (layout::xy_ilp::LAYER_WIDTH / 2.0);
        return vec![(mid_x, from_y), (mid_x, to_y)];
    }
    let available_space = layout::xy_ilp::LAYER_WIDTH - 20.0;
    let gap = available_space / (*size as f64 - 1.0);
    let x_start = from_x + (layout::xy_ilp::LAYER_WIDTH - available_space) / 2.0;
    let mid_x = x_start + (*index as f64 * gap);

    vec![(mid_x, from_y), (mid_x, to_y)]
}

fn get_data_right_port(data_node: &DataNode) -> (f64, f64) {
    let (width, height) = get_node_size(&data_node.datatype.clone().unwrap());
    let (x, y, x_offset, y_offset) = (
        data_node.x.unwrap(),
        data_node.y.unwrap(),
        data_node.x_offset.unwrap(),
        data_node.y_offset.unwrap(),
    );
    (
        x + width as f64 + x_offset,
        y + height as f64 / 2.0 + y_offset,
    )
}

fn get_data_left_port(data_node: &DataNode) -> (f64, f64) {
    let (_, height) = get_node_size(&data_node.datatype.clone().unwrap());
    let (x, y, x_offset, y_offset) = (
        data_node.x.unwrap(),
        data_node.y.unwrap(),
        data_node.x_offset.unwrap(),
        data_node.y_offset.unwrap(),
    );
    (x + x_offset, y + height as f64 / 2.0 + y_offset)
}

fn get_data_mid_point(
    index: &usize,
    size: &usize,
    from_node: &DataNode,
    to_node: &Node,
) -> Vec<(f64, f64)> {
    let (from_x, from_y) = get_data_right_port(from_node);
    let (_, to_y) = get_right_port(to_node);
    if *size == 1 {
        let mid_x = from_x + (layout::xy_ilp::LAYER_WIDTH / 2.0);
        return vec![(mid_x, from_y), (mid_x, to_y)];
    }
    let available_space = layout::xy_ilp::LAYER_WIDTH - 20.0;
    let gap = available_space / (*size as f64 - 1.0);
    let x_start = from_x + (layout::xy_ilp::LAYER_WIDTH - available_space) / 2.0;
    let mid_x = x_start + (*index as f64 * gap);

    vec![(mid_x, from_y), (mid_x, to_y)]
}

fn get_data_mid_point_reversed(
    index: &usize,
    size: &usize,
    from_node: &DataNode,
    to_node: &Node,
) -> Vec<(f64, f64)> {
    let (from_x, from_y) = get_right_port(to_node);
    let (_, to_y) = get_data_right_port(from_node);
    if *size == 1 {
        let mid_x = from_x + (layout::xy_ilp::LAYER_WIDTH / 2.0);
        return vec![(mid_x, from_y), (mid_x, to_y)];
    }
    let available_space = layout::xy_ilp::LAYER_WIDTH - 20.0;
    let gap = available_space / (*size as f64 - 1.0);
    let x_start = from_x + (layout::xy_ilp::LAYER_WIDTH - available_space) / 2.0;
    let mid_x = x_start + (*index as f64 * gap);

    vec![(mid_x, from_y), (mid_x, to_y)]
}
