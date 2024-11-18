use crate::common::graph::Graph;
use crate::common::bpmn_event::get_node_size;

/// Määrab servadele algus-, lõpp- ja painutuskohad.
pub fn assign_bend_points(graph: &mut Graph) {
    for edge in &mut graph.edges {
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from).unwrap();
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to).unwrap();

        let (from_x, from_y) = (from_node.x.unwrap(), from_node.y.unwrap() + from_node.y_offset.unwrap_or(0.0));
        let (to_x, to_y) = (to_node.x.unwrap(), to_node.y.unwrap() + to_node.y_offset.unwrap_or(0.0));
        let (from_width, from_height) = get_node_size(from_node.event.as_ref().unwrap());
        let (to_width, to_height) = get_node_size(to_node.event.as_ref().unwrap());

        // Keskpunktide arvutamine, arvestades offset'iga
        let from_center_x = from_x + from_width as f64 / 2.0;
        let from_center_y = from_y + from_height as f64 / 2.0;
        let to_center_x = to_x + to_width as f64 / 2.0;
        let to_center_y = to_y + to_height as f64 / 2.0;

        // Algus- ja lõpp-punktide määramine sõltuvalt suunast ja offset'ist
        let (edge_from_x, edge_from_y) = if to_center_x > from_center_x && to_center_y > from_center_y {
            // `to_node` asub paremal ja allpool
            (from_center_x, from_y + from_height as f64) // Algab alt keskelt
        } else if to_center_x > from_center_x && to_center_y < from_center_y {
            // `to_node` asub paremal ja üleval
            (from_x + from_width as f64, from_center_y) // Algab paremalt keskelt
        } else if to_center_x > from_center_x {
            // `to_node` asub otse paremal
            (from_x + from_width as f64, from_center_y) // Algab paremalt keskelt
        }  else if to_center_y > from_center_y {
            // `to_node` asub otse all
            (from_center_x, from_y + from_height as f64) // Algab alt keskelt
        } else {
            // `to_node` asub otse ülal
            (from_center_x, from_y) // Algab ülevalt keskelt
        };

        let (edge_to_x, edge_to_y) = if from_center_x < to_center_x && from_center_y < to_center_y {
            // `from_node` asub vasakul ja üleval
            (to_x, to_center_y) // Lõppeb ülevalt keskelt
        } else if from_center_x < to_center_x && from_center_y > to_center_y {
            // `from_node` asub vasakul ja allpool
            (to_center_x, to_y + to_height as f64) // Lõppeb alt keskelt
        } else if from_center_x < to_center_x {
            // `from_node` asub otse vasakul
            (to_x, to_center_y) // Lõppeb vasakult keskelt
        } else if from_center_y < to_center_y {
            // `from_node` asub otse ülal
            (to_center_x, to_y) // Lõppeb ülevalt keskelt
        } else {
            // `from_node` asub otse all
            (to_center_x, to_y + to_height as f64) // Lõppeb alt keskelt
        };



        // Paindepunktide määramine ainult vajaduse korral
        let mut bend_points = vec![];
        if (edge_from_x != edge_to_x) && (edge_from_y != edge_to_y) {
            if edge_from_x < edge_to_x && edge_from_y < edge_to_y {
                bend_points.push((edge_from_x, edge_to_y)); // Joondame esmalt x-koordinaadi järgi
            } else {
                bend_points.push((edge_to_x, edge_from_y)); // Joondame esmalt y-koordinaadi järgi
            }
        }

        // Salvestame kõik punktid järjekorras
        edge.adjusted_points = Some(vec![(edge_from_x, edge_from_y)]
            .into_iter()
            .chain(bend_points.into_iter())
            .chain(vec![(edge_to_x, edge_to_y)].into_iter())
            .collect());
    }
}
