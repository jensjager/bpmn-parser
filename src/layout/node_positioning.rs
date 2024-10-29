use crate::common::graph::Graph;
use std::collections::HashMap;

// Määrame sõlmede suurused ja vahed
const NODE_HEIGHT: f64 = 50.0;        // Sõlme kõrgus
const MIN_NODE_SPACING: f64 = 20.0;   // Minimaalne vertikaalne vahe sõlmede vahel
const NODE_WIDTH: f64 = 100.0;        // Sõlme laius
const LAYER_WIDTH: f64 = NODE_WIDTH + 50.0; // X-koordinaatide vahe (kihiti)

/// Määrab sõlmede X ja Y koordinaadid kihtide ja järjestuse põhjal.
pub fn assign_xy_to_nodes(graph: &mut Graph, layers: &Vec<(usize, i32)>) {
    let layer_width = LAYER_WIDTH; // Kasutame määratud kihivahet
    let node_spacing = NODE_HEIGHT + MIN_NODE_SPACING;  // Y-koordinaatide vahe sõlmede vahel (kihis)

    // Loome kaardi, et rühmitada sõlmed kihtide järgi
    let mut layer_map: HashMap<i32, Vec<usize>> = HashMap::new();
    for (node_id, layer) in layers {
        layer_map.entry(*layer).or_insert(vec![]).push(*node_id);
    }

    // Määrame igale kihile ja sõlmele koordinaadid
    for (layer, nodes) in layer_map.iter() {
        let x_start = 100.0; // X-koordinaadi alguspunkt
        let x = x_start + (*layer as f64 * layer_width); // X-koordinaat määratakse kihi numbri ja alguspunkti järgi
        //let x = *layer as f64 * layer_width; // X-koordinaat määratakse kihi numbri järgi
        let mut y_position = 100.0;

        // Sõlmede joondamine kihis
        for node_id in nodes {
            let y = y_position;
            y_position += node_spacing;

            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                // println!("Määran sõlmele {} koordinaadid: x = {}, y = {}", node_id, x, y);
                node.set_position(x, y);
            } else {
                println!("Sõlme {} ei leitud graafis", node_id);
            }
        }
    }
}
