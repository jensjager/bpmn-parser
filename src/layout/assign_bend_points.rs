use crate::common::graph::Graph;

/// Määrab servadele painutuspunktid ainult juhul, kui see on vajalik.
pub fn assign_bend_points(graph: &mut Graph) {
    for edge in &mut graph.edges {
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from).unwrap();
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to).unwrap();

        let mut bend_points = vec![];

        let x_diff = (from_node.x.unwrap() - to_node.x.unwrap()).abs();
        let y_diff = (from_node.y.unwrap() - to_node.y.unwrap()).abs();

        // Lisame painutuspunkti ainult siis, kui X- ja Y-koordinaadid ei ole samad
        if from_node.x != to_node.x && from_node.y != to_node.y {
            if x_diff > y_diff {
                // Kui X-vahe on suurem, liigume kõigepealt X-teljel ja siis Y-teljel
                bend_points.push((to_node.x.unwrap(), from_node.y.unwrap()));
            } else {
                // Kui Y-vahe on suurem, liigume kõigepealt Y-teljel ja siis X-teljel
                bend_points.push((from_node.x.unwrap(), to_node.y.unwrap()));
            }
        }

        // Lisame painutuspunktid ainult siis, kui need on olemas
        if !bend_points.is_empty() {
            for (x, y) in bend_points {
                edge.add_bend_point(x, y);
            }
        }
    }
}
