use crate::common::dataedge::DataEdge;
use crate::common::graph::{DataNodeId, Graph, NodeId};
use good_lp::*;

pub fn solve_data_layer_assignment(graph: &mut Graph) {
    for pool in graph.pools.iter_mut() {
        for lane in pool.lanes.iter_mut() {
            let mut data_nodes_in_lane: Vec<DataNodeId> = Vec::new();
            for data_node in graph.data_nodes.iter() {
                if data_node.lane.as_ref().unwrap() == lane.lane.as_ref().unwrap() {
                    data_nodes_in_lane.push(data_node.id);
                }
            }
            if data_nodes_in_lane.is_empty() {
                continue;
            }

            let lane_max_layer = graph.nodes[lane.nodes.last().unwrap().0]
                .clone()
                .layer_id
                .unwrap();

            let mut vars = variables!();

            let mut x_vars = Vec::new(); // D(v,u)
            let mut h_vars = Vec::new(); // h(v,l)

            for &data_node_id in &data_nodes_in_lane {
                for layer in 0..=lane_max_layer {
                    let x_var = vars.add(variable().binary());
                    x_vars.push((data_node_id, layer, x_var));

                    let h_var = vars.add(variable().binary());
                    h_vars.push((data_node_id, layer, h_var));
                }
            }

            let mut objective = Expression::from(0.0);

            for &data_node_id in &data_nodes_in_lane {
                let user_nodes = find_users_of_data_node(&graph.data_edges, data_node_id);
                let avg_layer: f64 = {
                    user_nodes
                        .iter()
                        .map(|user_id| graph.nodes[user_id.0].layer_id.unwrap() as f64)
                        .sum::<f64>()
                        / (user_nodes.len() as f64)
                };
                for user_id in user_nodes {
                    let user_layer = graph.nodes[user_id.0].layer_id.unwrap() as f64;

                    for (_, layer, x_var) in x_vars
                        .iter()
                        .filter(|(var_data_id, _, _)| *var_data_id == data_node_id)
                    {
                        let dist = (*layer as f64 - user_layer).abs();
                        objective = objective + dist * *x_var;

                        objective = objective + 0.2 * ((*layer as f64) - avg_layer).abs() * *x_var;
                    }

                    for (_, layer, h_var) in h_vars
                        .iter()
                        .filter(|(var_data_id, _, _)| *var_data_id == data_node_id)
                    {
                        let dist = ((*layer as f64 + 0.5) - user_layer).abs();
                        objective = objective + dist * *h_var;

                        objective = objective + 0.005 * *h_var;

                        objective = objective + 0.1 * ((*layer as f64) - avg_layer).abs() * *h_var;
                    }
                }
            }

            let mut problem = vars.minimise(objective).using(default_solver);

            for &data_node_id in &data_nodes_in_lane {
                let mut sum_expr = Expression::from(0.0);
                for (var_data_id, _, x_var) in &x_vars {
                    if *var_data_id == data_node_id {
                        sum_expr = sum_expr + x_var;
                    }
                }
                for (var_data_id, _, h_var) in &h_vars {
                    if *var_data_id == data_node_id {
                        sum_expr = sum_expr + h_var;
                    }
                }
                problem = problem.with(sum_expr.eq(1));
            }

            let solution = problem.solve().unwrap();

            for &data_node_id in &data_nodes_in_lane {
                let data_node = &mut graph.data_nodes[data_node_id.0];
                let (mut best_layer, mut used_half) = (None, false);

                for (var_data_id, layer, x_var) in &x_vars {
                    if *var_data_id == data_node_id {
                        if solution.value(*x_var) > 0.5 {
                            best_layer = Some(*layer);
                            used_half = false;
                            break;
                        }
                    }
                }
                if best_layer.is_none() {
                    for (var_data_id, layer, h_var) in &h_vars {
                        if *var_data_id == data_node_id {
                            if solution.value(*h_var) > 0.5 {
                                best_layer = Some(*layer);
                                used_half = true;
                                break;
                            }
                        }
                    }
                }

                if let Some(layer) = best_layer {
                    data_node.layer_id = Some(layer);
                    data_node.uses_half_layer = used_half;
                }
            }
        }
    }
}

fn find_users_of_data_node(data_edges: &Vec<DataEdge>, data_node_id: DataNodeId) -> Vec<NodeId> {
    data_edges
        .iter()
        .filter_map(|edge| {
            if edge.from == data_node_id {
                Some(edge.to)
            } else {
                None
            }
        })
        .collect()
}
