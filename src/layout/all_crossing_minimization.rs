use std::collections::HashMap;

use crate::common::graph::{Graph, NodeId};
use crate::common::pool::Pool;

#[derive(Debug, PartialEq)]
struct TempNode {
    id: usize,
    is_datanode: bool,
    layer_id: Option<usize>,
    avg: f64,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TempNodeId {
    id: usize,
    is_datanode: bool,
}

impl Ord for TempNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.avg != other.avg {
            self.avg
                .partial_cmp(&other.avg)
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            self.id.cmp(&other.id)
        }
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
    let mut prev_positions: HashMap<TempNodeId, Vec<usize>> = HashMap::new();
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
                layer_id: node.layer_id,
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
                layer_id: data_node.layer_id,
                avg: 0.0,
            });
            if data_node.pos_in_layer == None {
                data_node.pos_in_layer = Some(entry.len());
            }
        }
        println!("New go");
        for ((lane, _), nodes) in all_nodes.iter_mut() {
            for node in nodes.iter_mut() {
                find_average(node, lane, graph);
                println!("{} - {} - {}", node.id, node.avg, node.is_datanode);
                match &mut graph.nodes[node.id].event {
                    Some(crate::common::bpmn_event::BpmnEvent::ActivityTask(ref mut x)) => {
                        x.display_text = node.avg.to_string();
                    }
                    _ => (),
                }
            }

            nodes.sort();
        }
        for (_, nodes) in all_nodes.iter() {
            for (new_pos_in_layer, node) in nodes.iter().enumerate() {
                let temp_node = TempNodeId {
                    id: node.id,
                    is_datanode: node.is_datanode,
                };
                let entry = prev_positions.entry(temp_node).or_insert(vec![]);
                entry.push(new_pos_in_layer);
                if entry.iter().filter(|pos| **pos == new_pos_in_layer).count() > 3 {
                    break;
                }
                if node.is_datanode {
                    changed = true;
                    graph.data_nodes[node.id].pos_in_layer = Some(new_pos_in_layer);
                } else {
                    changed = true;
                    graph.nodes[node.id].pos_in_layer = Some(new_pos_in_layer);
                }
            }
        }
    }

    // place_single_data_nodes(graph);
}

fn place_single_data_nodes(graph: &mut Graph) {
    for data_node in graph.data_nodes.iter_mut() {
        let connections: Vec<_> = graph
            .data_edges
            .iter()
            .filter(|data_edge| data_edge.from == data_node.id)
            .collect();

        if connections.len() == 1 {
            let data_edge = connections[0];
            let to_node_pos = graph.nodes[data_edge.to.0].pos_in_layer.unwrap_or(0);
            data_node.pos_in_layer = Some(to_node_pos + 1);
            let layer_nodes: &Vec<NodeId> = {
                &graph
                    .pools
                    .iter_mut()
                    .find(|pool| pool.pool_name == data_node.pool)
                    .unwrap()
                    .lanes
                    .iter_mut()
                    .find(|lane| lane.lane == data_node.lane)
                    .unwrap()
                    .nodes
            };
            layer_nodes.iter().for_each(|node_id| {
                let node = &mut graph.nodes[node_id.0];
                if node.pos_in_layer >= data_node.pos_in_layer {
                    node.pos_in_layer = Some(node.pos_in_layer.unwrap() + 1);
                }
            });
        }
    }
}

fn find_average(temp_node: &mut TempNode, lane: &Option<String>, graph: &mut Graph) {
    let mut sum: usize = 0;
    let mut count: usize = 0;

    if !temp_node.is_datanode {
        for edge in graph.edges.iter() {
            if edge.to.0 == temp_node.id && !edge.temp_disabled {
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
                // Dont take same layer connections into account
                // Maybe TODO: crosslane same layer connections
                if temp_node.layer_id == from_node.layer_id {
                    continue;
                }
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
