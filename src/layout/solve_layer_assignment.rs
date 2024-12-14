use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::lane::Lane;
use crate::common::node::Node;
use crate::common::pool::Pool;
use good_lp::*;
use std::collections::HashMap;

pub fn solve_layer_assignment(graph: &mut Graph) {
    find_crossings(&graph.edges, &mut graph.pools);

    for pool in graph.pools.iter_mut() {
        for lane in pool.get_lanes_mut() {
            solve_layers(&graph.edges, lane);
        }
    }
}

fn solve_layers(edges: &Vec<Edge>, lane: &mut Lane) {
    let mut vars = variables!();
    let mut layer_vars = Vec::new();

    for node in lane.get_layers() {
        let layer_var = vars.add(variable().integer().min(0));
        layer_vars.push((node.id, layer_var));
    }

    let mut objective = Expression::from(0.0);
    for edge in edges {
        let from_var = layer_vars
            .iter()
            .find(|(id, _)| *id == edge.from)
            .map(|(_, v)| *v);
        let to_var = layer_vars
            .iter()
            .find(|(id, _)| *id == edge.to)
            .map(|(_, v)| *v);

        if let (Some(from_var), Some(to_var)) = (from_var, to_var) {
            objective = objective + (to_var - from_var);
        }
    }

    let mut problem = vars.minimise(objective).using(coin_cbc);
    problem.set_parameter("logLevel", "0");

    for edge in edges {
        let from_var = layer_vars
            .iter()
            .find(|(id, _)| *id == edge.from)
            .map(|(_, v)| *v);
        let to_var = layer_vars
            .iter()
            .find(|(id, _)| *id == edge.to)
            .map(|(_, v)| *v);

        if let (Some(from_var), Some(to_var)) = (from_var, to_var) {
            problem = problem.with((to_var - from_var).geq(1));
        }
    }

    let solution = problem.solve().unwrap();
    for (node_id, layer_var) in &layer_vars {
        let layer_value = solution.value(*layer_var) as usize;
        if let Some(node) = lane.get_layers_mut().iter_mut().find(|n| n.id == *node_id) {
            node.layer_id = Some(layer_value);
        }
    }

    lane.sort_nodes_by_layer_id();
}

fn find_crossings(edges: &Vec<Edge>, pools: &mut Vec<Pool>) {
    for pool in pools {
        let nodes_by_id: HashMap<usize, Node> = pool.get_nodes_by_id();
        for lane in pool.get_lanes_mut() {
            for node in lane.get_layers_mut() {
                let (crosses, found_to_node) = find_crossing(&nodes_by_id, edges, node);
                node.crosses_lanes = crosses;
                node.to_node_id = found_to_node;
            }
        }
    }
}

fn find_crossing(nodes_by_id: &HashMap<usize, Node>, edges: &Vec<Edge>, node: &Node) -> (bool, Option<usize>) {
    for edge in edges {
        if edge.from == node.id {
            let to_node = nodes_by_id.get(&edge.to);
            if let Some(to_node) = to_node {
                if to_node.lane.as_ref().unwrap() != node.lane.as_ref().unwrap() {
                    return (true, Some(edge.to));
                }
            } else {
                return (true, Some(edge.to));
            }
        }
    }
    (false, None)
}
