use crate::common::bpmn_event::{BpmnEvent};
use crate::common::bpmn_event::{get_node_size};
use crate::common::graph::Graph;
use crate::common::node::Node;
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
<bpmn:definitions xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
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
                // Gateways
                BpmnEvent::GatewayExclusive => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:exclusiveGateway id="Gateway_{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:exclusiveGateway>
"#,
                        node.id,
                        node.id - 1,
                        node.id
                    ));
                }
                BpmnEvent::GatewayInclusive => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:inclusiveGateway id="Gateway_{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:inclusiveGateway>
"#,
                        node.id,
                        node.id - 1,
                        node.id
                    ));
                }
                BpmnEvent::GatewayJoin(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:parallelGateway id="Gateway_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:parallelGateway>
"#,
                        node.id,
                        label,
                        node.id - 1,
                        node.id
                    ));
                }

                // Activities
                BpmnEvent::Middle(label) | BpmnEvent::ActivityTask(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:task id="Activity_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:task>
"#,
                        node.id,
                        label,
                        node.id - 1,
                        node.id
                    ));
                }
                BpmnEvent::ActivitySubprocess(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:subProcess id="SubProcess_{}" name="{}" triggeredByEvent="false">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:subProcess>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::ActivityCallActivity(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:callActivity id="CallActivity_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:callActivity>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::ActivityEventSubprocess(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:subProcess id="EventSubProcess_{}" name="{}" triggeredByEvent="true">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:subProcess>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::ActivityTransaction(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:transaction id="Transaction_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:transaction>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }

                // Start Events
                BpmnEvent::Start(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:startEvent id="StartEvent_{}" name="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:startEvent>
"#,
                        node.id, label, node.id
                    ));
                }
                BpmnEvent::StartTimerEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:startEvent id="StartEvent_{}" name="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:timerEventDefinition />
  </bpmn:startEvent>
"#,
                        node.id, label, node.id
                    ));
                }
                BpmnEvent::StartSignalEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:startEvent id="StartEvent_{}" name="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:signalEventDefinition />
  </bpmn:startEvent>
"#,
                        node.id, label, node.id
                    ));
                }
                BpmnEvent::StartMessageEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:startEvent id="StartEvent_{}" name="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:messageEventDefinition />
  </bpmn:startEvent>
"#,
                        node.id, label, node.id
                    ));
                }
                BpmnEvent::StartConditionalEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:startEvent id="StartEvent_{}" name="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:conditionalEventDefinition>
      <bpmn:condition xsi:type="bpmn:tFormalExpression">/* Your condition here */</bpmn:condition>
    </bpmn:conditionalEventDefinition>
  </bpmn:startEvent>
"#,
                        node.id, label, node.id
                    ));
                }

                // End Events
                BpmnEvent::End(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
  </bpmn:endEvent>
"#,
                        node.id,
                        label,
                        node.id - 1
                    ));
                }
                BpmnEvent::EndErrorEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:errorEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::EndCancelEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:cancelEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::EndSignalEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:signalEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::EndMessageEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:messageEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::EndTerminateEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:terminateEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::EndEscalationEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:escalationEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }
                BpmnEvent::EndCompensationEvent(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:endEvent id="EndEvent_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:compensateEventDefinition />
  </bpmn:endEvent>
"#,
                        node.id, label, node.id - 1
                    ));
                }

                // Boundary Events
                BpmnEvent::BoundaryEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundaryErrorEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:errorEventDefinition />
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundaryTimerEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:timerEventDefinition />
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundarySignalEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:signalEventDefinition />
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundaryMessageEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:messageEventDefinition />
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundaryEscalationEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:escalationEventDefinition />
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundaryConditionalEvent(label, attached_to, cancel_activity) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="{}">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:conditionalEventDefinition>
      <bpmn:condition xsi:type="bpmn:tFormalExpression">/* Your condition here */</bpmn:condition>
    </bpmn:conditionalEventDefinition>
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        if *cancel_activity { "true" } else { "false" },
                        node.id
                    ));
                }
                BpmnEvent::BoundaryCompensationEvent(label, attached_to) => {
                    let attached_to_ref = get_node_bpmn_id_by_id(*attached_to, graph);
                    // Compensation boundary events are always non-interrupting
                    bpmn.push_str(&format!(
                        r#"<bpmn:boundaryEvent id="BoundaryEvent_{}" name="{}" attachedToRef="{}" cancelActivity="false">
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
    <bpmn:compensateEventDefinition />
  </bpmn:boundaryEvent>
"#,
                        node.id,
                        label,
                        attached_to_ref,
                        node.id
                    ));
                }

                // Data Objects
                BpmnEvent::DataStoreReference(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:dataStoreReference id="DataStoreReference_{}" name="{}" />
"#,
                        node.id, label
                    ));
                }
                BpmnEvent::DataObjectReference(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:dataObjectReference id="DataObjectReference_{}" name="{}" />
"#,
                        node.id, label
                    ));
                }

                // Tasks
                BpmnEvent::TaskUser(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:userTask id="UserTask_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:userTask>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::TaskService(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:serviceTask id="ServiceTask_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:serviceTask>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::TaskBusinessRule(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:businessRuleTask id="BusinessRuleTask_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:businessRuleTask>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }
                BpmnEvent::TaskScript(label) => {
                    bpmn.push_str(&format!(
                        r#"<bpmn:scriptTask id="ScriptTask_{}" name="{}">
    <bpmn:incoming>Flow_{}</bpmn:incoming>
    <bpmn:outgoing>Flow_{}</bpmn:outgoing>
  </bpmn:scriptTask>
"#,
                        node.id, label, node.id - 1, node.id
                    ));
                }

                // Default case
                _ => {}
            }
        }
    }

    // Generate sequence flows
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

    // Add BPMN diagram elements (BPMNPlane and BPMNShape)
    bpmn.push_str(
        r#"
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
"#,
    );

    for node in &graph.nodes {
        let (width, height) = if let Some(event) = &node.event {
            get_node_size(event)
        } else {
            (100, 80) // Default size if event is None
        };

        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNShape id="{}_di" bpmnElement="{}">
          <dc:Bounds x="{:.2}" y="{:.2}" width="{}" height="{}" />
        </bpmndi:BPMNShape>
    "#,
            get_node_bpmn_id(node),
            get_node_bpmn_id(node),
            node.x.unwrap_or(0.0),
            node.y.unwrap_or(0.0),
            width,
            height
        ));
    }

    // Add BPMNEdge elements
    for edge in &graph.edges {
        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNEdge id="Flow_{}_{}_di" bpmnElement="Flow_{}_{}">
"#,
            edge.from, edge.to, edge.from, edge.to,
        ));

        // Use adjusted_points for waypoints
        if let Some(points) = &edge.adjusted_points {
            for &(x, y) in points {
                bpmn.push_str(&format!(
                    r#"<di:waypoint x="{:.2}" y="{:.2}" />"#,
                    x, y
                ));
            }
        }

        bpmn.push_str("</bpmndi:BPMNEdge>");
    }

    bpmn.push_str(
        r#"    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#,
    );

    bpmn
}

pub fn export_to_xml(bpmn: &String) {
    // Write BPMN to file
    let file_path = "generated_bpmn.bpmn";
    let mut file = File::create(file_path).expect("Unable to create file");
    file.write_all(bpmn.as_bytes())
        .expect("Unable to write data");

    println!("BPMN file generated at: {}", file_path);
}

fn get_node_bpmn_id(node: &Node) -> String {
    if let Some(event) = &node.event {
        match event {
            BpmnEvent::Start(_) => format!("StartEvent_{}", node.id),
            BpmnEvent::End(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::StartTimerEvent(_) => format!("StartEvent_{}", node.id),
            BpmnEvent::StartSignalEvent(_) => format!("StartEvent_{}", node.id),
            BpmnEvent::StartMessageEvent(_) => format!("StartEvent_{}", node.id),
            BpmnEvent::StartConditionalEvent(_) => format!("StartEvent_{}", node.id),
            BpmnEvent::EndErrorEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::EndCancelEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::EndSignalEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::EndMessageEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::EndTerminateEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::EndEscalationEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::EndCompensationEvent(_) => format!("EndEvent_{}", node.id),
            BpmnEvent::Middle(_) | BpmnEvent::ActivityTask(_) => format!("Activity_{}", node.id),
            BpmnEvent::ActivitySubprocess(_) => format!("SubProcess_{}", node.id),
            BpmnEvent::ActivityCallActivity(_) => format!("CallActivity_{}", node.id),
            BpmnEvent::ActivityEventSubprocess(_) => format!("EventSubProcess_{}", node.id),
            BpmnEvent::ActivityTransaction(_) => format!("Transaction_{}", node.id),
            BpmnEvent::TaskUser(_) => format!("UserTask_{}", node.id),
            BpmnEvent::TaskService(_) => format!("ServiceTask_{}", node.id),
            BpmnEvent::TaskBusinessRule(_) => format!("BusinessRuleTask_{}", node.id),
            BpmnEvent::TaskScript(_) => format!("ScriptTask_{}", node.id),
            BpmnEvent::GatewayExclusive => format!("Gateway_{}", node.id),
            BpmnEvent::GatewayInclusive => format!("Gateway_{}", node.id),
            BpmnEvent::GatewayJoin(_) => format!("Gateway_{}", node.id),
            BpmnEvent::BoundaryEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundaryErrorEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundaryTimerEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundarySignalEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundaryMessageEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundaryEscalationEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundaryConditionalEvent(_, _, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::BoundaryCompensationEvent(_, _) => format!("BoundaryEvent_{}", node.id),
            BpmnEvent::DataStoreReference(_) => format!("DataStoreReference_{}", node.id),
            BpmnEvent::DataObjectReference(_) => format!("DataObjectReference_{}", node.id),
            _ => format!("Node_{}", node.id),
        }
    } else {
        format!("Node_{}", node.id)
    }
}

fn get_node_bpmn_id_by_id(node_id: usize, graph: &Graph) -> String {
    let node = graph.nodes.iter().find(|n| n.id == node_id).unwrap();
    get_node_bpmn_id(node)
}
