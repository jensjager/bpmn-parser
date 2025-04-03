use crate::common::{
    bpmn_event::{get_node_size, BpmnEvent},
    graph::Graph,
};
use good_lp::*;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
struct TempNode {
    id: usize,
    is_datanode: bool,
    is_dummy: bool,
    pos_in_layer: usize,
    layer: usize,
    lane: Option<String>,
}

impl PartialOrd for TempNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.pos_in_layer.partial_cmp(&other.pos_in_layer)
    }
}

impl Ord for TempNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos_in_layer.cmp(&other.pos_in_layer)
    }
}

const STARTING_MIN_VALUE: f64 = 100.0;
const MIN_SPACE: f64 = 100.0;
const MIN_SPACE_IN_GATEWAY_LAYER: f64 = 50.0;
const DUMMY_MIN_SPACE: f64 = 100.0;
const EDGE_WEIGHT: f64 = 1.0;
const DATA_EDGE_WEIGHT: f64 = 0.01;
const DUMMY_EDGE_WEIGHT: f64 = 2.0;
const MIN_SPACE_BETWEEN_DUMMYS: f64 = 5.0;

pub fn assign_xy_ilp(graph: &mut Graph) {
    let mut vars = variables!();

    // Vec<(node_id, is_datanode, y_var)>
    let mut y_vars = Vec::new();

    let mut all_nodes: HashMap<(Option<String>, Option<usize>), Vec<TempNode>> = HashMap::new();
    for node in graph.nodes.iter_mut() {
        let lane_name = node.lane.clone();
        let layer_id = node.layer_id;
        let dummy = if node.event.clone().unwrap() == BpmnEvent::Dummy() {
            true
        } else {
            false
        };
        let entry = all_nodes.entry((lane_name, layer_id)).or_insert(Vec::new());
        entry.push(TempNode {
            id: node.id.0,
            is_datanode: false,
            is_dummy: dummy,
            pos_in_layer: node.pos_in_layer.unwrap(),
            layer: layer_id.unwrap(),
            lane: node.lane.clone(),
        });
        let y_var = vars.add(variable().integer().min(STARTING_MIN_VALUE));
        y_vars.push((node.id.0, false, y_var));
    }
    for data_node in graph.data_nodes.iter_mut() {
        let lane_name = data_node.lane.clone();
        let layer_id = data_node.layer_id;
        let entry = all_nodes.entry((lane_name, layer_id)).or_insert(Vec::new());
        entry.push(TempNode {
            id: data_node.id.0,
            is_datanode: true,
            is_dummy: false,
            pos_in_layer: data_node.pos_in_layer.unwrap(),
            layer: layer_id.unwrap(),
            lane: data_node.lane.clone(),
        });
        let y_var = vars.add(variable().integer().min(STARTING_MIN_VALUE));
        y_vars.push((data_node.id.0, true, y_var));
    }

    // Vec<(from_node_is_datanode, from_node_id, to_node_is_datanode, to_node_id, diff_var)>
    let mut diff_vars = Vec::new();

    let mut objective = Expression::from(0.0);

    for data_edge in graph.data_edges.iter() {
        let diff_var = vars.add(variable().min(0.0));
        diff_vars.push((true, data_edge.from.0, false, data_edge.to.0, diff_var));
        objective = objective + diff_var * DATA_EDGE_WEIGHT;
    }

    for edge in graph.edges.iter() {
        if edge.temp_disabled {
            continue;
        }
        let diff_var = vars.add(variable().min(0.0));
        diff_vars.push((false, edge.from.0, false, edge.to.0, diff_var));
        if edge.is_dummy {
            objective = objective + diff_var * DUMMY_EDGE_WEIGHT;
        } else {
            objective = objective + diff_var * EDGE_WEIGHT;
        }
    }

    let mut problem = vars.minimise(objective).using(default_solver);

    all_nodes.iter_mut().for_each(|(_, nodes)| {
        nodes.sort();
    });

    for (_, nodes) in all_nodes.iter() {
        if nodes.len() <= 1 {
            continue;
        }
        for i in 1..nodes.len() {
            let upper_node =
                get_y_var(&y_vars, &nodes[i - 1].id, &nodes[i - 1].is_datanode).unwrap();
            let lower_node = get_y_var(&y_vars, &nodes[i].id, &nodes[i].is_datanode).unwrap();
            if nodes[i].is_dummy {
                if nodes[i - 1].is_dummy {
                    problem = problem.with((lower_node - upper_node).geq(MIN_SPACE_BETWEEN_DUMMYS))
                } else {
                    problem = problem.with((lower_node - upper_node).geq(DUMMY_MIN_SPACE));
                }
            } else {
                problem = problem.with((lower_node - upper_node).geq(MIN_SPACE));
            }
        }

        for (i, node) in nodes.iter().enumerate() {
            if graph.nodes[node.id].event
                == Some(BpmnEvent::Gateway(crate::lexer::GatewayType::Exclusive))
            {
                let gateway_node = &graph.nodes[node.id];
                let prev_layer = all_nodes
                    .get(&(
                        gateway_node.lane.clone(),
                        Some(gateway_node.layer_id.unwrap() - 1),
                    ))
                    .unwrap();
                let highest = prev_layer.first().unwrap();
                let lowest = prev_layer.last().unwrap();
                let highest_var = get_y_var(&y_vars, &highest.id, &highest.is_datanode).unwrap();
                let lowest_var = get_y_var(&y_vars, &lowest.id, &lowest.is_datanode).unwrap();
                let next_layer = all_nodes
                    .get(&(
                        gateway_node.lane.clone(),
                        Some(gateway_node.layer_id.unwrap() + 1),
                    ))
                    .unwrap();
                let next_highest = next_layer.first().unwrap();
                let next_lowest = next_layer.last().unwrap();
                let next_highest_var =
                    get_y_var(&y_vars, &next_highest.id, &next_highest.is_datanode).unwrap();
                let next_lowest_var =
                    get_y_var(&y_vars, &next_lowest.id, &next_lowest.is_datanode).unwrap();
                for (j, node) in nodes.iter().enumerate() {
                    let node_var = get_y_var(&y_vars, &node.id, &node.is_datanode).unwrap();
                    if node.is_dummy {
                        continue;
                    }
                    if j < i {
                        problem =
                            problem.with((highest_var - node_var).geq(MIN_SPACE_IN_GATEWAY_LAYER))
                    } else if j > i {
                        problem =
                            problem.with((node_var - lowest_var).geq(MIN_SPACE_IN_GATEWAY_LAYER))
                    } else {
                        // problem = problem.with(((highest_var + lowest_var) / 2).eq(node_var));
                        // problem =
                        //     problem.with(((next_highest_var + next_lowest_var) / 2).eq(node_var));
                    }
                }
            }
        }
    }

    for (from_is_datanode, from, to_is_datanode, to, diff_var) in diff_vars.iter() {
        let from_var = get_y_var(&y_vars, from, from_is_datanode).unwrap();
        let to_var = get_y_var(&y_vars, to, to_is_datanode).unwrap();
        problem = problem.with((from_var - to_var).leq(diff_var));
        problem = problem.with((to_var - from_var).leq(diff_var));
    }

    let solution = problem.solve().unwrap();

    for node in graph.nodes.iter_mut() {
        let y_val = get_y_var(&y_vars, &node.id.0, &false).map(|var| solution.value(var));

        node.y = y_val;
    }
    for data_node in graph.data_nodes.iter_mut() {
        let y_val = get_y_var(&y_vars, &data_node.id.0, &true).map(|var| solution.value(var));

        data_node.y = y_val;
    }

    assign_x(graph);
    find_pools_lanes(graph);
}

fn get_y_var(
    y_vars: &[(usize, bool, good_lp::Variable)],
    node_id: &usize,
    is_data_node: &bool,
) -> Option<good_lp::Variable> {
    for (var_node_id, var_is_data_node, y_var) in y_vars.iter() {
        if var_node_id == node_id && var_is_data_node == is_data_node {
            return Some(y_var.clone());
        }
    }

    None
}

const INITIAL_X_OFFSET: f64 = 50.0;
const X_LAYER_WIDTH: f64 = 150.0;
const LARGEST_NODE_HEIGHT: usize = 80;
const LARGEST_NODE_WIDTH: usize = 100;

pub fn assign_x(graph: &mut Graph) {
    for node in graph.nodes.iter_mut() {
        node.x = Some(node.layer_id.unwrap() as f64 * X_LAYER_WIDTH + INITIAL_X_OFFSET);
        // TODO set offsets
        let (node_size_x, node_size_y) = get_node_size(node.event.as_ref().unwrap());
        let y_offset = if node_size_y < LARGEST_NODE_HEIGHT {
            (LARGEST_NODE_HEIGHT - node_size_y) as f64 / 2.0
        } else {
            0.0
        };
        let x_offset = if node_size_x < LARGEST_NODE_WIDTH {
            (LARGEST_NODE_WIDTH - node_size_x) as f64 / 2.0
        } else {
            0.0
        };
        node.x_offset = Some(x_offset);
        node.y_offset = Some(y_offset);
    }
    for data_node in graph.data_nodes.iter_mut() {
        data_node.x = Some(data_node.layer_id.unwrap() as f64 * X_LAYER_WIDTH + INITIAL_X_OFFSET);
        let (node_size_x, node_size_y) = get_node_size(data_node.datatype.as_ref().unwrap());
        let y_offset = if node_size_y < LARGEST_NODE_HEIGHT {
            (LARGEST_NODE_HEIGHT - node_size_y) as f64 / 2.0
        } else {
            0.0
        };
        let x_offset = if node_size_x < LARGEST_NODE_WIDTH {
            (LARGEST_NODE_WIDTH - node_size_x) as f64 / 2.0
        } else {
            0.0
        };
        data_node.x_offset = Some(x_offset);
        data_node.y_offset = Some(y_offset);
        if data_node.uses_half_layer {
            data_node.x_offset = Some(data_node.x_offset.unwrap() + 80.0);
        }
    }
}

const LAYER_HEIGHT: f64 = 180.0;
const DATA_NODE_LAYER_HEIGHT: f64 = 100.0;
pub const LAYER_WIDTH: f64 = 80.0;

fn find_pools_lanes(graph: &mut Graph) {
    let mut x = 0.0;
    let mut y = 80.0;

    for pool in graph.pools.iter_mut() {
        pool.x = Some(0.0);
        pool.y = Some(y);
        for lane in pool.lanes.iter_mut() {
            lane.x = Some(0.0);
            lane.y = Some(y);
            for node_id in lane.nodes.iter() {
                let node = &graph.nodes[node_id.0];
                if node.y > Some(y) {
                    y = node.y.unwrap() + LAYER_HEIGHT;
                }
                if node.x > Some(x) {
                    x = node.x.unwrap() + LAYER_WIDTH;
                }
            }
            for data_node in graph.data_nodes.iter() {
                if data_node.lane == lane.lane && data_node.pool == pool.pool_name {
                    if data_node.y > Some(y) {
                        y = data_node.y.unwrap() + DATA_NODE_LAYER_HEIGHT;
                    }
                    if data_node.x > Some(x) {
                        x = data_node.x.unwrap() + LAYER_WIDTH;
                    }
                }
            }
            lane.height = Some(y - lane.y.unwrap());
            lane.width = Some(lane.x.unwrap() + x);
        }
        pool.height = Some(y - pool.y.unwrap());
        pool.width = Some(pool.x.unwrap() + x);
    }
}
