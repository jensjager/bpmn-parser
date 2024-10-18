use std::collections::HashMap;
use crate::common::graph::Graph;

pub fn assign_xy_to_nodes(graph: &mut Graph, layers: &Vec<(usize, i32)>) {
    let layer_width = 150.0; // X-koordinaatide vahe (kihiti)
    let node_spacing = 100.0;  // Y-koordinaatide vahe sõlmede vahel (kihis)

    // Loome kaardi, et rühmitada sõlmed kihtide järgi
    let mut layer_map: HashMap<i32, Vec<usize>> = HashMap::new();
    for (node_id, layer) in layers {
        layer_map.entry(*layer).or_insert(vec![]).push(*node_id);
    }

    // Määrame igale kihile ja sõlmele koordinaadid
    for (layer, nodes) in layer_map.iter() {
        let x = *layer as f64 * layer_width; // X-koordinaat määratakse kihi numbri järgi
        let mut y_position = 0.0;

        // Sõlmede joondamine kihis
        for (index, node_id) in nodes.iter().enumerate() {
            let y = y_position; // Y-koordinaat määratakse sõlme järjestuse järgi kihis
            y_position += node_spacing; // Suurendame Y-koordinaati järgmise sõlme jaoks

            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                println!("Määran sõlmele {} koordinaadid: x = {}, y = {}", node_id, x, y);
                node.set_position(x, y);
            } else {
                println!("Sõlme {} ei leitud graafis", node_id);
            }
        }
    }
}
