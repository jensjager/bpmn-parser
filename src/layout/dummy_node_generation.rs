use std::collections::HashMap;

use crate::common::{
    graph::{DataNodeId, Graph, NodeId},
    pool::Pool,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TempEdge {
    from_id: usize,
    from_is_data_node: bool,
    to_id: usize,
    to_is_data_node: bool,
}

impl TempEdge {
    fn new(from_id: usize, from_is_data_node: bool, to_id: usize, to_is_data_node: bool) -> Self {
        TempEdge {
            from_id,
            from_is_data_node,
            to_id,
            to_is_data_node,
        }
    }
}

struct Context {
    from_pool: Option<String>,
    from_lane: Option<String>,
    to_pool: Option<String>,
    to_lane: Option<String>,
    from_id: usize,
    from_is_data_node: bool,
    to_id: usize,
    to_is_data_node: bool,
    steps_to_middle: usize,
    start_layer: usize,
    end_layer: usize,
    cur_pool: Option<String>,
    start_lane: Option<String>,
    end_lane: Option<String>,
}

pub fn generate_dummy_nodes(graph: &mut Graph) {
    let mut dummy_nodes: HashMap<TempEdge, Vec<(Option<String>, Option<String>, Option<usize>)>> =
        HashMap::new();
    generate_node_dummys(graph, &mut dummy_nodes);
    generate_data_node_dummys(graph, &mut dummy_nodes);

    create_dummy_nodes(graph, dummy_nodes);
}

fn generate_data_node_dummys(
    graph: &mut Graph,
    dummy_nodes: &mut HashMap<TempEdge, Vec<(Option<String>, Option<String>, Option<usize>)>>,
) {
    for data_edge in graph.data_edges.iter() {
        let (
            from_id,
            from_is_data_node,
            from_layer_id,
            from_pool,
            from_lane,
            to_id,
            to_is_data_node,
            to_layer_id,
            to_pool,
            to_lane,
        ) = if data_edge.is_reversed {
            let from_node = &graph.nodes[data_edge.to.0];
            let to_node = &graph.data_nodes[data_edge.from.0];
            (
                from_node.id.0,
                false,
                from_node.layer_id,
                from_node.pool.clone(),
                from_node.lane.clone(),
                to_node.id.0,
                true,
                to_node.layer_id,
                to_node.pool.clone(),
                to_node.lane.clone(),
            )
        } else {
            let from_node = &graph.data_nodes[data_edge.from.0];
            let to_node = &graph.nodes[data_edge.to.0];
            (
                from_node.id.0,
                true,
                from_node.layer_id,
                from_node.pool.clone(),
                from_node.lane.clone(),
                to_node.id.0,
                false,
                to_node.layer_id,
                to_node.pool.clone(),
                to_node.lane.clone(),
            )
        };
        let (start_layer, end_layer, cur_pool, start_lane, end_lane): (
            usize,
            usize,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = if from_layer_id < to_layer_id {
            (
                from_layer_id.unwrap(),
                to_layer_id.unwrap(),
                from_pool.clone(),
                from_lane.clone(),
                to_lane.clone(),
            )
        } else {
            (
                to_layer_id.unwrap(),
                from_layer_id.unwrap(),
                to_pool.clone(),
                to_lane.clone(),
                from_lane.clone(),
            )
        };

        let steps_to_middle = from_layer_id.unwrap().abs_diff(to_layer_id.unwrap()) / 2;

        let context = Context {
            from_pool,
            from_lane,
            to_pool,
            to_lane,
            from_id,
            from_is_data_node,
            to_id,
            to_is_data_node,
            steps_to_middle,
            start_layer,
            end_layer,
            cur_pool,
            start_lane,
            end_lane,
        };

        find_dummy_nodes(&graph.pools, dummy_nodes, context);
    }
}

fn generate_node_dummys(
    graph: &mut Graph,
    dummy_nodes: &mut HashMap<TempEdge, Vec<(Option<String>, Option<String>, Option<usize>)>>,
) {
    for edge in graph.edges.iter() {
        let from_node = &graph.nodes[edge.from.0];
        let to_node = &graph.nodes[edge.to.0];

        let steps_to_middle = from_node
            .layer_id
            .unwrap()
            .abs_diff(to_node.layer_id.unwrap())
            / 2;

        let (start_layer, end_layer, cur_pool, start_lane, end_lane): (
            usize,
            usize,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = if from_node.layer_id < to_node.layer_id {
            (
                from_node.layer_id.unwrap(),
                to_node.layer_id.unwrap(),
                from_node.pool.clone(),
                from_node.lane.clone(),
                to_node.lane.clone(),
            )
        } else {
            (
                to_node.layer_id.unwrap(),
                from_node.layer_id.unwrap(),
                to_node.pool.clone(),
                to_node.lane.clone(),
                from_node.lane.clone(),
            )
        };

        let context = Context {
            from_pool: from_node.pool.clone(),
            from_lane: from_node.lane.clone(),
            to_pool: to_node.pool.clone(),
            to_lane: to_node.lane.clone(),
            from_id: from_node.id.0,
            from_is_data_node: false,
            to_id: to_node.id.0,
            to_is_data_node: false,
            steps_to_middle,
            start_layer,
            end_layer,
            cur_pool,
            start_lane,
            end_lane,
        };

        find_dummy_nodes(&graph.pools, dummy_nodes, context);
    }
}

fn find_dummy_nodes(
    pools: &Vec<Pool>,
    dummy_nodes: &mut HashMap<TempEdge, Vec<(Option<String>, Option<String>, Option<usize>)>>,
    context: Context,
) {
    if context.from_pool == context.to_pool && context.from_lane != context.to_lane {
        let temp_edge = TempEdge::new(
            context.from_id,
            context.from_is_data_node,
            context.to_id,
            context.to_is_data_node,
        );
        dummy_nodes.insert(temp_edge.clone(), vec![]);
        let entry = dummy_nodes.get_mut(&temp_edge).unwrap();
        for i in context.start_layer + 1..context.start_layer + context.steps_to_middle {
            entry.push((
                context.cur_pool.clone(),
                context.start_lane.clone(),
                Some(i),
            ));
        }
        for i in context.start_layer + context.steps_to_middle + 1..context.end_layer {
            entry.push((context.cur_pool.clone(), context.end_lane.clone(), Some(i)));
        }
        for pool in pools.iter() {
            if pool.pool_name == context.cur_pool {
                let mut encounters: usize = 0;
                for lane in pool.lanes.iter() {
                    if lane.lane == context.start_lane || lane.lane == context.end_lane {
                        encounters += 1;
                    }
                    if encounters == 1 {
                        entry.push((
                            context.cur_pool.clone(),
                            lane.lane.clone(),
                            Some(context.start_layer + context.steps_to_middle),
                        ));
                    }
                    if encounters == 2 {
                        break;
                    }
                }
                break;
            }
        }
    } else if context.start_layer + 1 != context.end_layer {
        let temp_edge = TempEdge::new(
            context.from_id,
            context.from_is_data_node,
            context.to_id,
            context.to_is_data_node,
        );
        dummy_nodes.insert(temp_edge.clone(), vec![]);
        let entry = dummy_nodes.get_mut(&temp_edge).unwrap();
        for i in context.start_layer + 1..context.end_layer {
            entry.push((context.cur_pool.clone(), context.end_lane.clone(), Some(i)));
        }
    }
}

fn create_dummy_nodes(
    graph: &mut Graph,
    dummy_nodes: HashMap<TempEdge, Vec<(Option<String>, Option<String>, Option<usize>)>>,
) {
    for (temp_edge, dummy_list) in dummy_nodes {
        if dummy_list.is_empty() {
            continue;
        }
        if temp_edge.to_is_data_node || temp_edge.from_is_data_node {
            let (from_id, to_id, is_reversed) = if temp_edge.from_is_data_node {
                disable_data_edge(graph, temp_edge.from_id, temp_edge.to_id, false);
                (temp_edge.from_id, temp_edge.to_id, false)
            } else {
                disable_data_edge(graph, temp_edge.to_id, temp_edge.from_id, true);
                (temp_edge.to_id, temp_edge.from_id, true)
            };

            let (pool, lane, layer) = &dummy_list[0];
            let new_dummy_id = graph.add_dummy_node(pool.clone(), lane.clone(), *layer);
            // First dummy edge needs to be a data edge if data edge is not reversed
            if is_reversed {
                graph.add_dummy_edge_from_data_node(NodeId(to_id), new_dummy_id, from_id, to_id);
            } else {
                graph.add_dummy_data_edge(DataNodeId(from_id), new_dummy_id, from_id, to_id);
            }

            let mut prev_id = new_dummy_id;
            for (pool, lane, layer) in dummy_list.iter().skip(1) {
                let new_dummy_id = graph.add_dummy_node(pool.clone(), lane.clone(), *layer);
                graph.add_dummy_edge(prev_id, new_dummy_id, Some(from_id), Some(to_id), true);
                prev_id = new_dummy_id;
            }
            // Last dummy edge needs to be a data edge if data edge is reversed
            if is_reversed {
                graph.add_dummy_data_edge_reversed(DataNodeId(from_id), prev_id, from_id, to_id);
            } else {
                graph.add_dummy_edge_from_data_node(prev_id, NodeId(to_id), from_id, to_id);
            }
        } else {
            if let Some(edge) = graph
                .edges
                .iter_mut()
                .find(|edge| edge.from.0 == temp_edge.from_id && edge.to.0 == temp_edge.to_id)
            {
                edge.temp_disabled = true;
            }
            let mut prev_id = NodeId(temp_edge.from_id);
            for (pool, lane, layer) in dummy_list {
                let new_dummy_id = graph.add_dummy_node(pool, lane, layer);
                graph.add_dummy_edge(prev_id, new_dummy_id, None, None, false);
                prev_id = new_dummy_id;
            }
            graph.add_dummy_edge(prev_id, NodeId(temp_edge.to_id), None, None, false);
        }
    }
}

fn disable_data_edge(graph: &mut Graph, from_id: usize, to_id: usize, reversed: bool) {
    if let Some(data_edge) = graph.data_edges.iter_mut().find(|data_edge| {
        data_edge.from.0 == from_id && data_edge.to.0 == to_id && data_edge.is_reversed == reversed
    }) {
        data_edge.temp_disabled = true;
    }
}
