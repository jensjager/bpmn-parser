use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::common::bpmn_event::BpmnEvent;

use crate::layout::solve_layer_assignment::solve_layer_assignment;
use crate::layout::crossing_minimization::reduce_crossings;
use crate::layout::node_positioning::assign_xy_to_nodes;
use crate::layout::assign_bend_points::assign_bend_points;

use std::fs::File;
use std::io::Write;

// Adds color attributes for `stroke_color` and `fill_color` to the BPMN node XML if they exist.
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
                BpmnEvent::MessageEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:intermediateThrowEvent id="Event_{}" name="{}">
    <bpmn:messageEventDefinition />
  </bpmn:intermediateThrowEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::TimerEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:intermediateCatchEvent id="Event_{}" name="{}">
    <bpmn:timerEventDefinition />
  </bpmn:intermediateCatchEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::ConditionalEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:intermediateCatchEvent id="Event_{}" name="{}">
    <bpmn:conditionalEventDefinition />
  </bpmn:intermediateCatchEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::SignalEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:intermediateThrowEvent id="Event_{}" name="{}">
    <bpmn:signalEventDefinition />
  </bpmn:intermediateThrowEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::ErrorEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="ErrorEvent_{}" name="{}">
    <bpmn:errorEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::EscalationEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EscalationEvent_{}" name="{}">
    <bpmn:escalationEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::CompensateEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="CompensateEvent_{}" name="{}">
    <bpmn:compensateEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::TerminateEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="TerminateEvent_{}" name="{}">
    <bpmn:terminateEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label
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

        let stroke_color = node.stroke_color.as_ref();
        let fill_color = node.fill_color.as_ref();
        let color_attributes = add_color_attributes(stroke_color, fill_color);

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

    bpmn.push_str(
        r#"
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
"#,
    );

    bpmn.push_str(
        r#"    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#,
    );

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
            _ => format!("Event_{}", node.id),
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
            _ => (36, 36), // Default size for events
        }
    } else {
        (100, 80) // Default size for tasks
    }
}

pub fn perform_layout(graph: &mut Graph) {
    let layers = solve_layer_assignment(graph);
    let new_layers = reduce_crossings(graph, &layers);
    assign_xy_to_nodes(graph, &new_layers);
    assign_bend_points(graph);
}
