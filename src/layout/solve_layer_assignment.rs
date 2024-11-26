use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::lane::Lane;
use crate::common::layer::Layer;
use crate::common::node::Node;
use crate::common::pool::Pool;
use good_lp::*;

// HashMap<String, HashMap<String, HashMap<i32, Node>>>
// HashMap<Pool, HashMap<Lane, HashMap<Layer, Node>>>

pub fn solve_layer_assignment(graph: &mut Graph) -> Vec<Pool> {
    find_crossings(graph);
    let mut pools = create_pools(graph);
    create_lanes(&mut pools, graph);

    for pool in &mut pools {
        let pool_name = pool.get_pool_name().to_string();

        for lane in pool.get_lanes_mut() {
            solve_layers(graph, &pool_name, lane);
        }
    }

    pools
}

fn solve_layers(graph: &mut Graph, pool_name: &String, lane: &mut Lane) {
    let mut vars = variables!();
    let mut layer_vars = Vec::new();

    for node in &graph.nodes {
        if let Some(node_pool_name) = &node.pool {
            if node_pool_name == pool_name {
                if let Some(lane_name) = &node.lane {
                    if lane.get_lane() == lane_name {
                        let layer_var = vars.add(variable().integer().min(0));
                        layer_vars.push((node.id, layer_var));
                    }
                }
            }
        }
    }

    let mut objective = Expression::from(0.0);
    for edge in &graph.edges {
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

    for edge in &graph.edges {
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

    // Solve problem
    let solution = problem.solve().unwrap();
    let mut lane_layers: Vec<Layer> = Vec::new();
    for (node_id, layer_var) in &layer_vars {
        let layer_value = solution.value(*layer_var) as i32;
        if let Some(layer) = lane_layers
            .iter_mut()
            .find(|l| l.get_layer() == layer_value)
        {
            if let Some(node) = graph.get_node_by_id(*node_id) {
                layer.add_node(node.clone());
            }
        } else {
            if let Some(node) = graph.get_node_by_id(*node_id) {
                let new_layer = Layer::new(layer_value, vec![node.clone()]);
                lane_layers.push(new_layer);
            }
        }
    }

    // Sort layers by id
    lane_layers.sort_by_key(|layer| layer.get_layer());
    lane.get_layers_mut().extend(lane_layers);
}

fn create_lanes(pools: &mut Vec<Pool>, graph: &mut Graph) {
    for pool in pools.iter_mut() {
        for node in &graph.nodes {
            if let Some(pool_name) = &node.pool {
                if pool.get_pool_name() == pool_name {
                    if let Some(lane_name) = &node.lane {
                        if !pool
                            .get_lanes()
                            .iter()
                            .any(|lane| lane.get_lane() == lane_name)
                        {
                            let lane = Lane::new(lane_name.clone(), Vec::new());
                            pool.add_lane(lane);
                        }
                    }
                }
            }
        }
    }
}

fn create_pools(graph: &mut Graph) -> Vec<Pool> {
    let mut pools: Vec<Pool> = Vec::new();
    for node in &graph.nodes {
        if let Some(pool_name) = &node.pool {
            let mut found = false;

            for pool in &*pools {
                if pool.get_pool_name() == pool_name {
                    found = true;
                    break;
                }
            }

            if !found {
                let pool = Pool::new(pool_name.clone(), Vec::new());
                pools.push(pool);
            }
        }
    }

    pools
}

// Find all nodes with edges that cross lanes
fn find_crossings(graph: &mut Graph) {
    for i in 0..graph.nodes.len() {
        let (crosses, found_to_node) = {
            let nodes = &graph.nodes;
            let edges = &graph.edges;
            find_crossing(nodes, edges, &graph.nodes[i])
        };

        graph.nodes[i].crosses_lanes = crosses;
        graph.nodes[i].to_node_id = found_to_node;
    }
}

fn find_crossing(nodes: &Vec<Node>, edges: &Vec<Edge>, node: &Node) -> (bool, Option<usize>) {
    for edge in edges {
        if edge.from == node.id {
            for node in nodes {
                if edge.to == node.id {
                    let to_node = node;
                    if to_node.pool != node.pool {
                        return (true, Some(to_node.id));
                    }
                }
            }
        }
    }

    (false, None)
}
