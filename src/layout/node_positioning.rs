use crate::common::{bpmn_event, graph::Graph};
use std::collections::HashMap;

// Määrame sõlmede suurused ja vahed
const MIN_NODE_SPACING: f64 = 20.0;   // Minimaalne vertikaalne vahe sõlmede vahel
const NODE_WIDTH: f64 = 100.0;        // Sõlme laius
const LAYER_WIDTH: f64 = NODE_WIDTH + 50.0; // X-koordinaatide vahe (kihiti)

/// Määrab sõlmede X ja Y koordinaadid kihtide ja järjestuse põhjal.
pub fn assign_xy_to_nodes(graph: &mut Graph, layers: &Vec<(usize, i32)>) {
    let layer_width = LAYER_WIDTH; // Kasutame määratud kihivahet
    
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
        let mut y_position = 200.0;
        
        // Sõlmede joondamine kihis
        for node_id in nodes {
            let node = graph.nodes.iter().find(|n| n.id == *node_id);
            let node_event = &node.unwrap().event.as_ref().unwrap();
            let (_, node_size_y) = bpmn_event::get_node_size(node_event);
            let mut node_spacing =  node_size_y as f64 + MIN_NODE_SPACING;  // Y-koordinaatide vahe sõlmede vahel (kihis)
            if node_spacing < 100.0 {
                node_spacing += 100.0 - node_spacing;
                println!("Väiksem kui 100: {}", node_spacing);
            }
            let y_offset = if node_size_y < 80 {
                (80 - node_size_y) as f64 / 2.0
            } else {
                0.0
            };
            let y = y_position;
            y_position += node_spacing;

            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                println!("Määran sõlmele {} koordinaadid: x = {}, y = {}, y_offset = {}", node_id, x, y, y_offset);
                node.set_position(x, y, y_offset);
            } else {
                println!("Sõlme {} ei leitud graafis", node_id);
            }
        }
    }
}
