pub mod common;
pub mod layout;

use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::common::bpmn_event::BpmnEvent;

use crate::layout::solve_layer_assignment::solve_layer_assignment;
use crate::layout::crossing_minimization::reduce_crossings;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub fn generate_bpmn(graph: &Graph, positions: &HashMap<usize, (f64, f64)>) -> String {
    let mut bpmn = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
xmlns:di="http://www.omg.org/spec/DD/20100524/DI"
xmlns:modeler="http://camunda.org/schema/modeler/1.0" id="Definitions_1"
targetNamespace="http://bpmn.io/schema/bpmn" exporter="Camunda Modeler"
exporterVersion="5.17.0">
  <bpmn:process id="Process_1" isExecutable="true">
"#,
    );

    // Create nodes
    for node in &graph.nodes {
        if let Some(event) = &node.event {
            match event {
                BpmnEvent::Start(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:startEvent id="StartEvent_{}" name="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:startEvent>
"#,
                        node.id, label, node.id
                    ));
                }
                BpmnEvent::Middle(label) | BpmnEvent::ActivityTask(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:task id="Activity_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:task>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::End(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::GatewayExclusive => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:exclusiveGateway id="Gateway_{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:exclusiveGateway>
"#,
                        node.id, node.id - 1, node.id
                    ));
                }
                BpmnEvent::GatewayJoin(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:parallelGateway id="Gateway_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:parallelGateway>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
            }
        }
    }

    // Create sequence flows
    for edge in &graph.edges {
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from).unwrap();
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to).unwrap();

        let source_ref = get_node_bpmn_id(from_node);
        let target_ref = get_node_bpmn_id(to_node);

        bpmn.push_str(&format!(
            r#"<bpmn:sequenceFlow id="Flow_{}_{}" sourceRef="{}" targetRef="{}" />
"#,
            edge.from, edge.to, source_ref, target_ref
        ));
    }

    bpmn.push_str(r#"  </bpmn:process>"#);

    // Add BPMN diagram details
    bpmn.push_str(
        r#"
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
"#,
    );

    // Define node sizes based on event types
    let node_sizes: Vec<(usize, usize)> = graph
        .nodes
        .iter()
        .map(|node| {
            if let Some(event) = &node.event {
                match event {
                    BpmnEvent::Start(_) | BpmnEvent::End(_) => (36, 36),
                    BpmnEvent::Middle(_) | BpmnEvent::ActivityTask(_) => (100, 80),
                    BpmnEvent::GatewayExclusive | BpmnEvent::GatewayJoin(_) => (50, 50),
                }
            } else {
                (100, 80) // Default size
            }
        })
        .collect();

    // Add BPMN shapes for nodes using calculated positions
    for (i, node) in graph.nodes.iter().enumerate() {
        let (x, y) = positions.get(&node.id).cloned().unwrap_or((0.0, 0.0));
        let (width, height) = node_sizes[i];

        let bpmn_element_id = get_node_bpmn_id(node);

        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNShape id="{}_di" bpmnElement="{}">
      <dc:Bounds x="{:.2}" y="{:.2}" width="{}" height="{}" />
      <bpmndi:BPMNLabel />
    </bpmndi:BPMNShape>
"#,
            bpmn_element_id, bpmn_element_id, x, y, width, height
        ));
    }

    // Add BPMN edges for sequence flows with adjusted waypoints
    for edge in &graph.edges {
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from).unwrap();
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to).unwrap();

        let (from_x, from_y) = positions.get(&from_node.id).cloned().unwrap_or((0.0, 0.0));
        let (to_x, to_y) = positions.get(&to_node.id).cloned().unwrap_or((0.0, 0.0));

        let (from_width, from_height) = get_node_size(from_node);
        let (to_width, to_height) = get_node_size(to_node);

        let (edge_from_x, edge_from_y) = (
            from_x + from_width as f64 / 2.0,
            from_y + from_height as f64 / 2.0,
        );

        let (edge_to_x, edge_to_y) = (
            to_x + to_width as f64 / 2.0,
            to_y + to_height as f64 / 2.0,
        );

        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNEdge id="Flow_{}_{}_di" bpmnElement="Flow_{}_{}">
  <di:waypoint x="{:.2}" y="{:.2}" />
  <di:waypoint x="{:.2}" y="{:.2}" />
</bpmndi:BPMNEdge>
"#,
            edge.from,
            edge.to,
            edge.from,
            edge.to,
            edge_from_x,
            edge_from_y,
            edge_to_x,
            edge_to_y
        ));
    }

    bpmn.push_str(
        r#"    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#,
    );

    // Write BPMN to file (optional)
    let file_path = "generated_bpmn.bpmn";
    let mut file = File::create(file_path).expect("Unable to create file");
    file.write_all(bpmn.as_bytes())
        .expect("Unable to write data");

    println!("BPMN file generated at: {}", file_path);

    bpmn
}

fn get_node_bpmn_id(node: &Node) -> String {
    if let Some(event) = &node.event {
        match event {
            BpmnEvent::Start(_) => format!("StartEvent_{}", node.id),
            BpmnEvent::End(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::Middle(_) | BpmnEvent::ActivityTask(_) => format!("Activity_{}", node.id),
            BpmnEvent::GatewayExclusive => format!("Gateway_{}", node.id),
            BpmnEvent::GatewayJoin(_) => format!("Gateway_{}", node.id),
        }
    } else {
        format!("Node_{}", node.id)
    }
}

fn get_node_size(node: &Node) -> (usize, usize) {
    if let Some(event) = &node.event {
        match event {
            BpmnEvent::Start(_) | BpmnEvent::End(_) => (36, 36),
            BpmnEvent::Middle(_) | BpmnEvent::ActivityTask(_) => (100, 80),
            BpmnEvent::GatewayExclusive | BpmnEvent::GatewayJoin(_) => (50, 50),
        }
    } else {
        (100, 80) // Default size
    }
}

pub fn perform_layout(graph: &Graph) -> HashMap<usize, (f64, f64)> {
    // Assign layers
    let layers = solve_layer_assignment(graph);
    // Reduce crossings
    let new_layers = reduce_crossings(graph, &layers);

    // Assign positions
    let positions = assign_positions(&new_layers);

    positions
}

fn assign_positions(layers: &Vec<(usize, i32)>) -> HashMap<usize, (f64, f64)> {
    use std::collections::HashMap;

    let mut positions = HashMap::new();

    // Group nodes by layer
    let mut layer_map: HashMap<i32, Vec<usize>> = HashMap::new();
    for &(node_id, layer) in layers {
        layer_map.entry(layer).or_insert(Vec::new()).push(node_id);
    }

    // Sort layers
    let mut sorted_layers: Vec<i32> = layer_map.keys().cloned().collect();
    sorted_layers.sort();

    // Assign positions
    let layer_spacing = 150.0;
    let node_spacing = 150.0;

    for (layer_index, layer) in sorted_layers.iter().enumerate() {
        let nodes_in_layer = &layer_map[layer];
        for (node_index, &node_id) in nodes_in_layer.iter().enumerate() {
            let x = node_index as f64 * node_spacing;
            let y = layer_index as f64 * layer_spacing;
            positions.insert(node_id, (x, y));
        }
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::bpmn_event::BpmnEvent;
    use crate::common::edge::Edge;
    use crate::common::graph::Graph;
    use crate::common::node::Node;

    #[test]
    fn test_generate_bpmn_with_multiple_middle_events() {
        let mut graph = Graph::new(vec![], vec![]);

        // Create nodes with BpmnEvent
        let start_node =
            Node::new(0, None, Some(BpmnEvent::Start("Start Event".to_string())));
        let middle_node1 = Node::new(
            1,
            None,
            Some(BpmnEvent::ActivityTask("Task 1".to_string())),
        );
        let middle_node2 = Node::new(
            2,
            None,
            Some(BpmnEvent::ActivityTask("Task 2".to_string())),
        );
        let middle_node3 = Node::new(
            3,
            None,
            Some(BpmnEvent::ActivityTask("Task 3".to_string())),
        );
        let end_node = Node::new(4, None, Some(BpmnEvent::End("End Event".to_string())));

        graph.add_node(start_node);
        graph.add_node(middle_node1);
        graph.add_node(middle_node2);
        graph.add_node(middle_node3);
        graph.add_node(end_node);

        // Create edges
        graph.add_edge(Edge::new(0, 1, None));
        graph.add_edge(Edge::new(1, 2, None));
        graph.add_edge(Edge::new(2, 3, None));
        graph.add_edge(Edge::new(3, 4, None));

        // Perform layout
        let positions = perform_layout(&graph);

        // Generate BPMN
        let bpmn_xml = generate_bpmn(&graph, &positions);
    }
}
