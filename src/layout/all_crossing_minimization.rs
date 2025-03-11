use std::collections::HashMap;

use crate::common::{graph::Graph, pool::Pool};

#[derive(Debug, PartialEq)]
struct TempNode {
    id: usize,
    is_datanode: bool,
    avg: f64,
}

impl Ord for TempNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.avg
            .partial_cmp(&other.avg)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for TempNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TempNode {}

pub fn reduce_all_crossings(graph: &mut Graph) {
    let mut changed = true;
    while changed {
        changed = false;

        // HashMap<(lane name, layer id), Vec<(node id, is datanode, calculated average in layer)>>
        let mut all_nodes: HashMap<(Option<String>, Option<usize>), Vec<TempNode>> = HashMap::new();
        for node in graph.nodes.iter_mut() {
            let lane_name = node.lane.clone();
            let layer_id = node.layer_id;
            let entry = all_nodes.entry((lane_name, layer_id)).or_insert(Vec::new());
            entry.push(TempNode {
                id: node.id.0,
                is_datanode: false,
                avg: 0.0,
            });
            if node.pos_in_layer == None {
                node.pos_in_layer = Some(entry.len());
            }
        }
        for data_node in graph.data_nodes.iter_mut() {
            let lane_name = data_node.lane.clone();
            let layer_id = data_node.layer_id;
            let entry = all_nodes.entry((lane_name, layer_id)).or_insert(Vec::new());
            entry.push(TempNode {
                id: data_node.id.0,
                is_datanode: true,
                avg: 0.0,
            });
            if data_node.pos_in_layer == None {
                data_node.pos_in_layer = Some(entry.len());
            }
        }
        for ((lane, _), nodes) in all_nodes.iter_mut() {
            for node in nodes.iter_mut() {
                find_average(node, lane, graph)
            }

            nodes.sort();
        }
        for (_, nodes) in all_nodes.iter() {
            for (new_pos_in_layer, node) in nodes.iter().enumerate() {
                if node.is_datanode {
                    if graph.data_nodes[node.id].pos_in_layer != Some(new_pos_in_layer) {
                        changed = true;
                    }
                    graph.data_nodes[node.id].pos_in_layer = Some(new_pos_in_layer);
                } else {
                    if graph.nodes[node.id].pos_in_layer != Some(new_pos_in_layer) {
                        changed = true;
                    }
                    graph.nodes[node.id].pos_in_layer = Some(new_pos_in_layer);
                }
            }
        }
    }
}

fn find_average(temp_node: &mut TempNode, lane: &Option<String>, graph: &mut Graph) {
    let mut sum: usize = 0;
    let mut count: usize = 0;

    if !temp_node.is_datanode {
        for edge in graph.edges.iter() {
            if edge.to.0 == temp_node.id {
                let from_node = &graph.nodes[edge.from.0];
                count += 1;
                if from_node.lane == *lane {
                    sum += from_node.pos_in_layer.unwrap();
                } else if past_to_node_lane(&from_node.lane, &lane, &graph.pools) {
                    temp_node.avg = f64::MAX;
                    return;
                } else {
                    temp_node.avg = f64::MIN;
                    return;
                }
            }
        }
        for data_edge in graph.data_edges.iter() {
            if data_edge.to.0 == temp_node.id {
                let from_data_node = &graph.data_nodes[data_edge.from.0];
                count += 1;
                if from_data_node.lane == *lane {
                    sum += from_data_node.pos_in_layer.unwrap();
                } else if past_to_node_lane(&from_data_node.lane, &lane, &graph.pools) {
                    temp_node.avg = f64::MAX;
                    return;
                } else {
                    temp_node.avg = f64::MIN;
                    return;
                }
            }
        }
    } else {
        for data_edge in graph.data_edges.iter() {
            if data_edge.from.0 == temp_node.id && data_edge.is_reversed {
                let from_node = &graph.nodes[data_edge.to.0];
                count += 1;
                if from_node.lane == *lane {
                    sum += from_node.pos_in_layer.unwrap();
                } else if past_to_node_lane(&from_node.lane, &lane, &graph.pools) {
                    temp_node.avg = f64::MAX;
                    return;
                } else {
                    temp_node.avg = f64::MIN;
                    return;
                }
            }
        }
    }

    if count > 0 {
        temp_node.avg = sum as f64 / count as f64;
    } else {
        temp_node.avg = 0.0;
    }
}

fn past_to_node_lane(
    from_lane: &Option<String>,
    to_lane: &Option<String>,
    pools: &Vec<Pool>,
) -> bool {
    for pool in pools.iter() {
        for lane in pool.lanes.iter() {
            if lane.lane == *from_lane {
                return true;
            } else if lane.lane == *to_lane {
                return false;
            }
        }
    }

    false
}
