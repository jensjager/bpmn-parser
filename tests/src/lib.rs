mod edge;
mod graph;
mod node;

use crate::edge::Edge;
use graph::Graph;
use crate::node::Node;
use crate::node::NodeType;

use std::fs::File;
use std::io::Write;
use rand::Rng;



pub fn generate_bpmn(graph: &Graph) -> String {
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

    // Creating nodes
    for node in &graph.nodes {
        match node.node_type {
            NodeType::StartEvent => {
                bpmn.push_str(&format!(
                    r#"<bpmn:startEvent id="StartEvent_{}" name="Start Event">
      <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    </bpmn:startEvent>
"#,
                    node.id, node.id
                ));
            }
            NodeType::IntermediateEvent => {
                bpmn.push_str(&format!(
                    r#"<bpmn:task id="Activity_{}" name="Middle Event">
      <bpmn:incoming>Flow_{}</bpmn:incoming>
      <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    </bpmn:task>
"#,
                    node.id, node.id - 1, node.id
                ));
            }
            NodeType::EndEvent => {
                bpmn.push_str(&format!(
                    r#"<bpmn:endEvent id="EndEvent_{}" name="End Event">
      <bpmn:incoming>Flow_{}</bpmn:incoming>
    </bpmn:endEvent>
"#,
                    node.id, node.id - 1
                ));
            }
        }
    }

    // Creating sequence flows
    for edge in &graph.edges {
        bpmn.push_str(&format!(
            r#"<bpmn:sequenceFlow id="Flow_{}" sourceRef="{}" targetRef="{}" />
"#,
            edge.from(),
            match graph.nodes[edge.from()].node_type {
                NodeType::StartEvent => format!("StartEvent_{}", edge.from()),
                NodeType::IntermediateEvent => format!("Activity_{}", edge.from()),
                NodeType::EndEvent => format!("EndEvent_{}", edge.from()),
            },
            match graph.nodes[edge.to()].node_type {
                NodeType::StartEvent => format!("StartEvent_{}", edge.to()),
                NodeType::IntermediateEvent => format!("Activity_{}", edge.to()),
                NodeType::EndEvent => format!("EndEvent_{}", edge.to()),
            }
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

    // Define node sizes
    let node_sizes: Vec<(usize, usize)> = graph.nodes.iter().map(|node| {
        match node.node_type {
            NodeType::StartEvent | NodeType::EndEvent => (36, 36),  // Width and height for events
            NodeType::IntermediateEvent => (100, 80),               // Width and height for tasks
        }
    }).collect();

    // Calculate positions for nodes with random y-coordinate
    let mut rng = rand::thread_rng();
    let node_positions: Vec<(usize, usize)> = graph.nodes.iter().enumerate().map(|(i, _node)| {
        let spacing = 200;  // Minimum spacing between nodes on x-axis
        let x = 150 + i * spacing;
        let y = rng.gen_range(0..=400);  // Random y-coordinate between 0 and 400
        (x, y)
    }).collect();

    // Add BPMN shapes for nodes using calculated positions
    for (i, node) in graph.nodes.iter().enumerate() {
        let (x, y) = node_positions[i];
        let (width, height) = node_sizes[i];
        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNShape id="{}_di" bpmnElement="{}">
      <dc:Bounds x="{}" y="{}" width="{}" height="{}" />
      <bpmndi:BPMNLabel />
    </bpmndi:BPMNShape>
"#,
            match node.node_type {
                NodeType::StartEvent => format!("StartEvent_{}", node.id),
                NodeType::IntermediateEvent => format!("Activity_{}", node.id),
                NodeType::EndEvent => format!("EndEvent_{}", node.id),
            },
            match node.node_type {
                NodeType::StartEvent => format!("StartEvent_{}", node.id),
                NodeType::IntermediateEvent => format!("Activity_{}", node.id),
                NodeType::EndEvent => format!("EndEvent_{}", node.id),
            },
            x, y, width, height
        ));
    }

    // Add BPMN edges for sequence flows with adjusted waypoints
    for edge in &graph.edges {
        let (from_x, from_y) = node_positions[edge.from()];
        let (to_x, to_y) = node_positions[edge.to()];

        // Get the sizes of the source and target nodes
        let (from_width, from_height) = node_sizes[edge.from()];
        let (to_width, to_height) = node_sizes[edge.to()];

        // Calculate the center points of the source and target nodes
        let from_center_x = from_x as f64 + from_width as f64 / 2.0;
        let from_center_y = from_y as f64 + from_height as f64 / 2.0;

        let to_center_x = to_x as f64 + to_width as f64 / 2.0;
        let to_center_y = to_y as f64 + to_height as f64 / 2.0;

        // Determine the horizontal and vertical distances
        let dx = to_center_x - from_center_x;
        let dy = to_center_y - from_center_y;
        let angle = dy.atan2(dx);

        // Determine the node types
        let from_node_type = &graph.nodes[edge.from()].node_type;
        let to_node_type = &graph.nodes[edge.to()].node_type;

        // Initialize edge start and end points
        let (edge_from_x, edge_from_y) = match from_node_type {
            NodeType::StartEvent | NodeType::EndEvent => {
                // Circles: Calculate point on circumference
                let radius = from_width as f64 / 2.0;
                (
                    from_center_x + radius * angle.cos(),
                    from_center_y + radius * angle.sin(),
                )
            }
            NodeType::IntermediateEvent => {
                // Rectangles: Connect to left or right edge at vertical center
                if dx >= 0.0 {
                    // Target is to the right
                    (from_x as f64 + from_width as f64, from_center_y)
                } else {
                    // Target is to the left
                    (from_x as f64, from_center_y)
                }
            }
        };

        let (edge_to_x, edge_to_y) = match to_node_type {
            NodeType::StartEvent | NodeType::EndEvent => {
                // Circles: Calculate point on circumference
                let radius = to_width as f64 / 2.0;
                let angle_opposite = angle + std::f64::consts::PI;
                (
                    to_center_x + radius * angle_opposite.cos(),
                    to_center_y + radius * angle_opposite.sin(),
                )
            }
            NodeType::IntermediateEvent => {
                // Rectangles: Connect to left or right edge at vertical center
                if dx >= 0.0 {
                    // Coming from the left
                    (to_x as f64, to_center_y)
                } else {
                    // Coming from the right
                    (to_x as f64 + to_width as f64, to_center_y)
                }
            }
        };

        // Add the BPMN edge with adjusted waypoints
        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNEdge id="Flow_{}_di" bpmnElement="Flow_{}">
      <di:waypoint x="{:.2}" y="{:.2}" />
      <di:waypoint x="{:.2}" y="{:.2}" />
    </bpmndi:BPMNEdge>
"#,
            edge.from(),
            edge.from(),
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
    file.write_all(bpmn.as_bytes()).expect("Unable to write data");

    println!("BPMN file generated at: {}", file_path);

    bpmn
}





#[cfg(test)]
mod tests {
    use super::*;
    use edge::Edge;
    use graph::Graph;
    use node::{Node, NodeType};

    #[test]
    fn test_generate_bpmn_with_multiple_middle_events() {
        let mut graph = Graph::new();

        // Creating nodes
        let start_node = Node::new(0, NodeType::StartEvent, Some(1));
        let middle_node1 = Node::new(1, NodeType::IntermediateEvent, Some(2));
        let middle_node2 = Node::new(2, NodeType::IntermediateEvent, Some(3));
        let middle_node3 = Node::new(3, NodeType::IntermediateEvent, Some(4));
        let end_node = Node::new(4, NodeType::EndEvent, Some(5));

        graph.add_node(start_node);
        graph.add_node(middle_node1);
        graph.add_node(middle_node2);
        graph.add_node(middle_node3);
        graph.add_node(end_node);

        // Creating edges
        let edge1 = Edge::new(0, 1);
        let edge2 = Edge::new(1, 2);
        let edge3 = Edge::new(2, 3);
        let edge4 = Edge::new(3, 4);

        graph.add_edge(edge1);
        graph.add_edge(edge2);
        graph.add_edge(edge3);
        graph.add_edge(edge4);

        // Generating BPMN
        let bpmn_xml = generate_bpmn(&graph);

        // Expected BPMN XML (updated to include additional middle event)
        let expected_bpmn = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" xmlns:di="http://www.omg.org/spec/DD/20100524/DI"
xmlns:modeler="http://camunda.org/schema/modeler/1.0" id="Definitions_1" targetNamespace="http://bpmn.io/schema/bpmn"
exporter="Camunda Modeler" exporterVersion="5.17.0">
  <bpmn:process id="Process_1" isExecutable="true">
    <bpmn:startEvent id="StartEvent_0" name="Start Event">
      <bpmn:outgoing>Flow_0</bpmn:outgoing>
    </bpmn:startEvent>
    <bpmn:task id="Activity_1" name="Middle Event">
      <bpmn:incoming>Flow_0</bpmn:incoming>
      <bpmn:outgoing>Flow_1</bpmn:outgoing>
    </bpmn:task>
    <bpmn:task id="Activity_2" name="Middle Event">
      <bpmn:incoming>Flow_1</bpmn:incoming>
      <bpmn:outgoing>Flow_2</bpmn:outgoing>
    </bpmn:task>
    <bpmn:endEvent id="EndEvent_3" name="End Event">
      <bpmn:incoming>Flow_2</bpmn:incoming>
    </bpmn:endEvent>
    <bpmn:sequenceFlow id="Flow_0" sourceRef="StartEvent_0" targetRef="Activity_1" />
    <bpmn:sequenceFlow id="Flow_1" sourceRef="Activity_1" targetRef="Activity_2" />
    <bpmn:sequenceFlow id="Flow_2" sourceRef="Activity_2" targetRef="EndEvent_3" />
  </bpmn:process>
  <!-- BPMN diagram elements (shapes and edges) would go here -->
</bpmn:definitions>
"#;
        //
        // // Use the compare_bpmn_structures function to test if the structure is the same
        // assert!(compare_bpmn_structures(&bpmn_xml, &expected_bpmn));
    }



    // fn compare_bpmn_structures(generated_bpmn: &str, expected_bpmn: &str) -> bool {
    //     // Parse the XML documents using roxmltree
    //     let generated_doc = Document::parse(generated_bpmn).expect("Failed to parse generated BPMN");
    //     let expected_doc = Document::parse(expected_bpmn).expect("Failed to parse expected BPMN");
    //
    //     // Get the process nodes from both documents
    //     let generated_process = generated_doc.descendants().find(|n| n.tag_name().name() == "process").expect("No process found in generated BPMN");
    //     let expected_process = expected_doc.descendants().find(|n| n.tag_name().name() == "process").expect("No process found in expected BPMN");
    //
    //     // Compare start events
    //     let generated_start_event = generated_process.descendants().find(|n| n.tag_name().name() == "startEvent").expect("No start event in generated BPMN");
    //     let expected_start_event = expected_process.descendants().find(|n| n.tag_name().name() == "startEvent").expect("No start event in expected BPMN");
    //
    //     if generated_start_event.attribute("name") != expected_start_event.attribute("name") {
    //         return false;
    //     }
    //
    //     // Compare tasks
    //     let generated_task = generated_process.descendants().find(|n| n.tag_name().name() == "task").expect("No task in generated BPMN");
    //     let expected_task = expected_process.descendants().find(|n| n.tag_name().name() == "task").expect("No task in expected BPMN");
    //
    //     if generated_task.attribute("name") != expected_task.attribute("name") {
    //         return false;
    //     }
    //
    //     // Compare end events
    //     let generated_end_event = generated_process.descendants().find(|n| n.tag_name().name() == "endEvent").expect("No end event in generated BPMN");
    //     let expected_end_event = expected_process.descendants().find(|n| n.tag_name().name() == "endEvent").expect("No end event in expected BPMN");
    //
    //     if generated_end_event.attribute("name") != expected_end_event.attribute("name") {
    //         return false;
    //     }
    //
    //     // Compare sequence flows (connections between elements)
    //     let generated_flows: Vec<_> = generated_process.descendants().filter(|n| n.tag_name().name() == "sequenceFlow").collect();
    //     let expected_flows: Vec<_> = expected_process.descendants().filter(|n| n.tag_name().name() == "sequenceFlow").collect();
    //
    //     // Ensure that both BPMN structures have the same number of flows
    //     if generated_flows.len() != expected_flows.len() {
    //         return false;
    //     }
    //
    //     // Compare each sequence flow
    //     for (gen_flow, exp_flow) in generated_flows.iter().zip(expected_flows.iter()) {
    //         // Compare sourceRef and targetRef (ignoring ids)
    //         if gen_flow.attribute("sourceRef") != exp_flow.attribute("sourceRef") ||
    //             gen_flow.attribute("targetRef") != exp_flow.attribute("targetRef") {
    //             return false;
    //         }
    //     }
    //
    //     // If everything matches, return true
    //     true
    // }

}