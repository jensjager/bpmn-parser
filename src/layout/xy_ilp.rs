use crate::common::graph::Graph;
use good_lp::*;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
struct TempNode {
    id: usize,
    is_datanode: bool,
    pos_in_layer: usize,
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

const MIN_SPACE: i32 = 100;

pub fn assign_xy_ilp(graph: &mut Graph) {
    let mut vars = variables!();

    let mut y_vars = Vec::new();

    let mut all_nodes: HashMap<(Option<String>, Option<usize>), Vec<TempNode>> = HashMap::new();
    for node in graph.nodes.iter_mut() {
        let lane_name = node.lane.clone();
        let layer_id = node.layer_id;
        let entry = all_nodes.entry((lane_name, layer_id)).or_insert(Vec::new());
        entry.push(TempNode {
            id: node.id.0,
            is_datanode: false,
            pos_in_layer: node.pos_in_layer.unwrap(),
        });
        let y_var = vars.add(variable().integer().min(100));
        y_vars.push((node.id.0, false, y_var));
    }
    for data_node in graph.data_nodes.iter_mut() {
        let lane_name = data_node.lane.clone();
        let layer_id = data_node.layer_id;
        let entry = all_nodes.entry((lane_name, layer_id)).or_insert(Vec::new());
        entry.push(TempNode {
            id: data_node.id.0,
            is_datanode: true,
            pos_in_layer: data_node.pos_in_layer.unwrap(),
        });
        let y_var = vars.add(variable().integer().min(100));
        y_vars.push((data_node.id.0, true, y_var));
    }

    let mut diff_vars = Vec::new();

    let mut objective = Expression::from(0.0);

    for data_edge in graph.data_edges.iter() {
        let diff_var = vars.add(variable().min(0));
        diff_vars.push((true, data_edge.from.0, data_edge.to.0, diff_var));
        objective = objective + diff_var;
    }

    for edge in graph.edges.iter() {
        let diff_var = vars.add(variable().min(0));
        diff_vars.push((false, edge.from.0, edge.to.0, diff_var));
        objective = objective + diff_var;
    }

    let mut problem = vars.minimise(objective).using(default_solver);

    for (_, nodes) in all_nodes.iter_mut() {
        nodes.sort();
        dbg!(&nodes);
        for i in 1..nodes.len() {
            let upper_node =
                get_y_var(&y_vars, &nodes[i - 1].id, &nodes[i - 1].is_datanode).unwrap();
            let lower_node = get_y_var(&y_vars, &nodes[i].id, &nodes[i].is_datanode).unwrap();
            problem = problem.with((lower_node - upper_node).geq(MIN_SPACE));
        }
    }
    for (is_datanode, from, to, diff_var) in diff_vars.iter() {
        let from_var = get_y_var(&y_vars, from, is_datanode).unwrap();
        let to_var = get_y_var(&y_vars, to, &false).unwrap();
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

pub fn assign_x(graph: &mut Graph) {
    for node in graph.nodes.iter_mut() {
        node.x = Some(node.layer_id.unwrap() as f64 * 120 as f64 + 100.0);
        node.x_offset = Some(0.0);
        node.y_offset = Some(0.0);
    }
    for data_node in graph.data_nodes.iter_mut() {
        data_node.x = Some(data_node.layer_id.unwrap() as f64 * 120 as f64 + 100.0);
        data_node.x_offset = Some(0.0);
        if data_node.uses_half_layer {
            data_node.x_offset = Some(80.0);
        }
        data_node.y_offset = Some(0.0);
    }
}

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
                    y = node.y.unwrap() + 180.0;
                }
                if node.x > Some(x) {
                    x = node.x.unwrap() + 80.0;
                }
            }
            for data_node in graph.data_nodes.iter() {
                if data_node.lane == lane.lane && data_node.pool == pool.pool_name {
                    if data_node.y > Some(y) {
                        dbg!("helo");
                        y = data_node.y.unwrap() + 100.0;
                    }
                    if data_node.x > Some(x) {
                        x = data_node.x.unwrap() + 80.0;
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
