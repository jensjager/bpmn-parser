// use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::common::bpmn_event::{self, BpmnEvent};
use std::fs::File;
use std::io::Write;

/// Adds color attributes for `stroke_color` and `fill_color` to the BPMN node XML if they exist.
fn add_color_attributes(stroke_color: Option<&String>, fill_color: Option<&String>) -> String {
    let stroke = stroke_color
        .map(|color| format!(r#" bioc:stroke="{}""#, color))
        .unwrap_or_default();
    let fill = fill_color
        .map(|color| format!(r#" bioc:fill="{}""#, color))
        .unwrap_or_default();
    format!("{}{}", stroke, fill)
}

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

    // Создаем узлы
    for node in &graph.nodes {
        let stroke_color = node.stroke_color.as_ref();
        let fill_color = node.fill_color.as_ref();
        let color_attributes = add_color_attributes(stroke_color, fill_color);

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

        // Добавляем `BPMNShape` с атрибутами цвета
        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNShape id="{}_di" bpmnElement="{}"{}>
      <dc:Bounds x="{:.2}" y="{:.2}" width="{}" height="{}" />
      <bpmndi:BPMNLabel />
    </bpmndi:BPMNShape>
"#,
            get_node_bpmn_id(node),
            get_node_bpmn_id(node),
            color_attributes,
            node.x.unwrap_or(0.0),
            node.y.unwrap_or(0.0),
            get_node_size(node).0,
            get_node_size(node).1
        ));
    }

    // Создаем последовательные потоки
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

    // Добавляем детали диаграммы BPMN
    bpmn.push_str(
        r#"
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
"#,
    );

    // Define node sizes based on event types
    // Defineeri sõlmede suurused otse
    let default_node_size = (100, 80);
    let gateway_size = (50, 50);
    let event_size = (36, 36);

    let node_sizes: Vec<(usize, usize)> = graph
        .nodes
        .iter()
        .map(|node| match &node.event {
            Some(BpmnEvent::Start(_)) | Some(BpmnEvent::End(_)) => event_size,
            Some(BpmnEvent::GatewayExclusive) | Some(BpmnEvent::GatewayJoin(_)) => gateway_size,
            Some(BpmnEvent::Middle(_)) | Some(BpmnEvent::ActivityTask(_)) => default_node_size,
            _ => default_node_size,
        })
        .collect();



    // Add BPMN shapes for nodes using calculated positions
    for (i, node) in graph.nodes.iter().enumerate() {
        let x = node.x.unwrap_or(0.0);
        let y = node.y.unwrap_or(0.0);
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

        let (from_x, from_y) = (from_node.x.unwrap_or(0.0), from_node.y.unwrap_or(0.0));
        let (to_x, to_y) = (to_node.x.unwrap_or(0.0), to_node.y.unwrap_or(0.0));

        let (from_width, from_height) = get_node_size(from_node);
        let (_to_width, to_height) = get_node_size(to_node);

        let (edge_from_x, edge_from_y) = (
            from_x + (from_width as f64), // Võtame alguspunkti X-koordinaadi, nihutades seda poole laiuse võrra
            from_y + (from_height as f64) / 2.0 // Võtame keskjoone
        );

        let (edge_to_x, edge_to_y) = (
            to_x, // Võtame lõpp-punkti X-koordinaadi, nihutades seda poole laiuse võrra
            to_y + (to_height as f64) / 2.0 // Võtame keskjoone
        );


        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNEdge id="Flow_{}_{}_di" bpmnElement="Flow_{}_{}">
  <di:waypoint x="{:.2}" y="{:.2}" />"#,
            edge.from,
            edge.to,
            edge.from,
            edge.to,
            edge_from_x,
            edge_from_y,
        ));

        // Bend points
        for &(x, y) in &edge.bend_points {
            bpmn.push_str(&format!(
                r#"
  <di:waypoint x="{:.2}" y="{:.2}" />"#,
                x, y
            ));
        }

        // End waypoint
        bpmn.push_str(&format!(
            r#"
  <di:waypoint x="{:.2}" y="{:.2}" />
</bpmndi:BPMNEdge>
"#,
            edge_to_x, edge_to_y
        ));
    }

    bpmn.push_str(
        r#"    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#,
    );

    // Записываем BPMN в файл (опционально)
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

pub(crate) fn get_node_size(node: &Node) -> (usize, usize) {
    if let Some(event) = &node.event {
        bpmn_event::get_node_size(event)
    } else {
        (100, 80) // Default size
    } 
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::common::bpmn_event::BpmnEvent;
//     use crate::common::edge::Edge;
//     use crate::common::graph::Graph;
//     use crate::layout::solve_layer_assignment::solve_layer_assignment;
//     use crate::layout::crossing_minimization::reduce_crossings;
//     use crate::layout::node_positioning::assign_xy_to_nodes;
//     use crate::layout::assign_bend_points::assign_bend_points;


//     fn perform_layout(graph: &mut Graph) {
//         // Assign layers
//         let layers = solve_layer_assignment(graph);
//         // Reduce crossings
//         let new_layers = reduce_crossings(graph, &layers);
    
//         // Assign positions to nodes using the imported function
//         assign_xy_to_nodes(graph, &new_layers);
    
//         // Assign bend points to edges using the imported function
//         assign_bend_points(graph);
//     }
    

//     #[test]
//     fn test_generate_bpmn_with_multiple_middle_events() {
//         let mut graph = Graph::new(vec![], vec![]);

//         let start_event = BpmnEvent::Start("Start Event".to_string());
//         let middle_event = BpmnEvent::ActivityTask("Task 1".to_string());
//         let middle_event2 = BpmnEvent::ActivityTask("Task 2".to_string());
//         let middle_event3 = BpmnEvent::ActivityTask("Task 3".to_string());
//         let end_event = BpmnEvent::End("End Event".to_string());

//         graph.add_node_noid(start_event);
//         graph.add_node_noid(middle_event);
//         graph.add_node_noid(middle_event2);
//         graph.add_node_noid(middle_event3);
//         graph.add_node_noid(end_event);

//         // Create edges
//         graph.add_edge(Edge::new(0, 1, None));
//         graph.add_edge(Edge::new(1, 2, None));
//         graph.add_edge(Edge::new(2, 3, None));
//         graph.add_edge(Edge::new(3, 4, None));

//         // Perform layout
//         perform_layout(&mut graph);

//         // Generate BPMN
//         let _bpmn_xml = generate_bpmn(&graph);

//         // Optionally, assert on `bpmn_xml` or inspect the output file.
//     }
// }
