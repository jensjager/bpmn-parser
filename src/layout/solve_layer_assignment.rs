use crate::common::graph::Graph;
use good_lp::variables;

pub fn solve_layer_assignment(graph: &Graph) -> Vec<(usize, i32)> {
    use good_lp::*;

    // Loome muutujad iga sõlme kihi jaoks
    let mut vars = variables!();

    let mut layer_vars = Vec::new();
    for node in &graph.nodes {
        let layer_var = vars.add(variable().integer().min(0)); // Kihid algavad 0-st
        layer_vars.push((node.id, layer_var));
    }

    // Minimeerime servade kogupikkust
    let mut objective = Expression::from(0.0);
    for edge in &graph.edges {
        let from_var = layer_vars.iter().find(|(id, _)| *id == edge.from).unwrap().1;
        let to_var = layer_vars.iter().find(|(id, _)| *id == edge.to).unwrap().1;

        objective = objective + (to_var - from_var);
    }



    let mut problem = vars.minimise(objective).using(coin_cbc);

    problem.set_parameter("logLevel", "0");

    // Lisa piirangud iga serva jaoks: L_v >= L_u + 1
    for edge in &graph.edges {
        let from_var = layer_vars.iter().find(|(id, _)| *id == edge.from).unwrap().1;
        let to_var = layer_vars.iter().find(|(id, _)| *id == edge.to).unwrap().1;

        problem = problem.with((to_var - from_var).geq(1));
    }

    let solution = problem.solve().unwrap();

    let mut layers = Vec::new();
    for (node_id, layer_var) in layer_vars {
        let layer_value = solution.value(layer_var) as i32;
        layers.push((node_id, layer_value));
    }

    layers
}
