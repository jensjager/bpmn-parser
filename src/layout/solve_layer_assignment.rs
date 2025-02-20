use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::lane::Lane;
use crate::common::node::Node;
use good_lp::*;

pub fn solve_layer_assignment(graph: &mut Graph) {
    for pool in &mut graph.pools {
        for lane in &mut pool.lanes {
            solve_layers(&mut graph.nodes, &graph.edges, lane);
        }
    }
}

fn solve_layers(nodes: &mut [Node], edges: &[Edge], lane: &mut Lane) {
    let mut vars = variables!();
    let mut layer_vars = Vec::new();

    let max_layer = lane.nodes.len() as f64;

    for node_id in &lane.nodes {
        let layer_var = vars.add(variable().integer().min(0).max(max_layer));
        layer_vars.push((node_id, layer_var));
    }

    let mut objective = Expression::from(0.0);
    for edge in edges {
        let from_var = layer_vars
            .iter()
            .find(|(id, _)| **id == edge.from)
            .map(|(_, v)| *v);
        let to_var = layer_vars
            .iter()
            .find(|(id, _)| **id == edge.to)
            .map(|(_, v)| *v);

        if let (Some(from_var), Some(to_var)) = (from_var, to_var) {
            objective = objective + (to_var - from_var);
        }
    }

    let mut problem = vars.minimise(objective).using(default_solver);
    for edge in edges {
        let from_var = layer_vars
            .iter()
            .find(|(id, _)| **id == edge.from)
            .map(|(_, v)| *v);
        let to_var = layer_vars
            .iter()
            .find(|(id, _)| **id == edge.to)
            .map(|(_, v)| *v);

        if let (Some(from_var), Some(to_var)) = (from_var, to_var) {
            problem = problem.with((to_var - from_var).geq(1));
        }
    }

    let solution = problem.solve().unwrap();
    for (node_id, layer_var) in &layer_vars {
        let layer_value = solution.value(*layer_var) as usize;
        nodes[node_id.0].layer_id = Some(layer_value);
    }

    lane.sort_nodes_by_layer_id(nodes);
}
