// to_xml.rs

use crate::common::bpmn_event::BpmnEvent;
use crate::common::edge::Edge;
use crate::common::graph::Graph;
use crate::common::lane::Lane;
use crate::common::node::Node;
use crate::common::pool::Pool;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

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
"#,
    );

    // Begin collaboration
    bpmn.push_str(r#"  <bpmn:collaboration id="Collaboration_1">"#);

    // Collect unique pool IDs from nodes
    let pool_ids: HashSet<String> = graph
        .pools
        .iter()
        .map(|pool| pool.get_pool_name())
        .collect();

    for pool_id in &pool_ids {
        bpmn.push_str(&format!(
            r#"<bpmn:participant id="Participant_{}" name="{}" processRef="Process_{}" />"#,
            pool_id, pool_id, pool_id
        ));
    }

    bpmn.push_str(r#"  </bpmn:collaboration>"#);

    // Generate processes for each pool
    for pool_id in &pool_ids {
        bpmn.push_str(&format!(
            r#"<bpmn:process id="Process_{}" isExecutable="true">"#,
            pool_id
        ));

        // Get nodes in this pool
        let pool_nodes: Vec<&Node> = graph.get_nodes_by_pool_name(pool_id);

        // Collect unique lane IDs within this pool
        let lane_ids: HashSet<String> = pool_nodes
            .iter()
            .filter_map(|node| node.lane.clone())
            .collect();

        // Generate laneSet if there are lanes
        if !lane_ids.is_empty() {
            bpmn.push_str(&format!(
                r#"<bpmn:laneSet id="LaneSet_{}">"#,
                pool_id
            ));

            for lane_id in &lane_ids {
                bpmn.push_str(&format!(
                    r#"<bpmn:lane id="Lane_{}" name="{}">"#,
                    lane_id, lane_id
                ));

                // Get nodes in this lane
                let lane_nodes: Vec<&Node> = pool_nodes
                    .iter()
                    .filter(|node| node.lane.as_deref() == Some(lane_id.as_str()))
                    .cloned()
                    .collect();

                // Add flowNodeRefs
                for node in &lane_nodes {
                    let node_id = get_node_bpmn_id(node);
                    bpmn.push_str(&format!(
                        r#"<bpmn:flowNodeRef>{}</bpmn:flowNodeRef>"#,
                        node_id
                    ));
                }

                bpmn.push_str(r#"</bpmn:lane>"#);
            }

            bpmn.push_str(r#"</bpmn:laneSet>"#);
        }

        // Generate flow nodes (events, tasks, etc.)
        for node in &pool_nodes {
            generate_flow_node(&mut bpmn, node, graph);
        }

        // Generate sequence flows
        generate_sequence_flows(&mut bpmn, &graph, &pool_nodes);

        bpmn.push_str(r#"</bpmn:process>"#);
    }

    // Add BPMN diagram elements
    bpmn.push_str(
        r#"<bpmndi:BPMNDiagram id="BPMNDiagram_1">
  <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Collaboration_1">
"#,
    );

    // Add BPMN shapes for participants (pools)
    for pool in graph.get_pools() {
        let pool_id = pool.get_pool_name();
        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNShape id="Participant_{}_di" bpmnElement="Participant_{}" isHorizontal="true">
    <dc:Bounds x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" />
  </bpmndi:BPMNShape>"#,
  pool_id,
  pool_id,
            /* x */ pool.x.unwrap_or(0.0),
            /* y */ pool.y.unwrap_or(0.0),
            /* width */ pool.width.unwrap_or(0.0),
            /* height */ pool.height.unwrap_or(0.0),
        )); 

        for lane in pool.get_lanes() {
            let lane_id = lane.get_lane();
            bpmn.push_str(&format!(
                r#"<bpmndi:BPMNShape id="Lane_{}_di" bpmnElement="Lane_{}" isHorizontal="true">
    <dc:Bounds x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" />
  </bpmndi:BPMNShape>"#,
                lane_id,
                lane_id,
                /* x */ lane.x.unwrap_or(0.0),
                /* y */ lane.y.unwrap_or(0.0),
                /* width */ lane.width.unwrap_or(0.0),
                /* height */ lane.height.unwrap_or(0.0),
            ));
        }
    }


    // Add BPMN shapes for flow nodes
    for pool in &graph.pools {
        for lane in pool.get_lanes() {
            for node in lane.get_layers() {
                let (width, height) = if let Some(event) = &node.event {
                    get_node_size(event)
                } else {
                    (100, 80)
                };

                let x = node.x.unwrap_or(0.0);
                let y = node.y.unwrap_or(0.0) + node.y_offset.unwrap_or(0.0);

                bpmn.push_str(&format!(
                r#"<bpmndi:BPMNShape id="{}_di" bpmnElement="{}">
                <dc:Bounds x="{:.2}" y="{:.2}" width="{}" height="{}" />
                </bpmndi:BPMNShape>"#,
                get_node_bpmn_id(node),
                get_node_bpmn_id(node),
                x,
                y,
                width,
                height
                ));
    }
}
    }

    // Add BPMN edges for sequence flows
    for edge in &graph.edges {
        bpmn.push_str(&format!(
            r#"<bpmndi:BPMNEdge id="Flow_{}_{}_di" bpmnElement="Flow_{}_{}">"#,
            edge.from, edge.to, edge.from, edge.to
        ));

        // Lisa waypoints adjusted_points-st
        if let Some(points) = &edge.adjusted_points {
            for (x, y) in points {
                bpmn.push_str(&format!(
                    r#"<di:waypoint x="{:.2}" y="{:.2}" />"#,
                    x, y
                ));
            }
        } else {
            eprintln!("Warning: Edge {} -> {} has no adjusted_points.", edge.from, edge.to);
        }

        bpmn.push_str(r#"</bpmndi:BPMNEdge>"#);
    }

    // Sulge BPMNPlane, BPMNDiagram ja definitions lõpus
    bpmn.push_str(
        r#"  </bpmndi:BPMNPlane>
</bpmndi:BPMNDiagram>
</bpmn:definitions>
"#,
    );

    bpmn
}

fn generate_flow_node(bpmn: &mut String, node: &Node, graph: &Graph) {
    if let Some(event) = &node.event {
        match event {
            // Start Events
            BpmnEvent::Start(label)
            | BpmnEvent::StartTimerEvent(label)
            | BpmnEvent::StartSignalEvent(label)
            | BpmnEvent::StartMessageEvent(label)
            | BpmnEvent::StartConditionalEvent(label) => {
                bpmn.push_str(&format!(
                    r#"<bpmn:startEvent id="{}" name="{}">"#,
                    get_node_bpmn_id(node),
                    label
                ));

                // Add outgoing flows
                for edge in graph.edges.iter().filter(|e| e.from == node.id) {
                    bpmn.push_str(&format!(
                        r#"<bpmn:outgoing>Flow_{}_{}"</bpmn:outgoing>"#,
                        edge.from, edge.to
                    ));
                }

                bpmn.push_str(r#"</bpmn:startEvent>"#);
            }

            // End Events
            BpmnEvent::End(label)
            | BpmnEvent::EndErrorEvent(label)
            | BpmnEvent::EndCancelEvent(label)
            | BpmnEvent::EndSignalEvent(label)
            | BpmnEvent::EndMessageEvent(label)
            | BpmnEvent::EndTerminateEvent(label)
            | BpmnEvent::EndEscalationEvent(label)
            | BpmnEvent::EndCompensationEvent(label) => {
                bpmn.push_str(&format!(
                    r#"<bpmn:endEvent id="{}" name="{}">"#,
                    get_node_bpmn_id(node),
                    label
                ));

                // Add incoming flows
                for edge in graph.edges.iter().filter(|e| e.to == node.id) {
                    bpmn.push_str(&format!(
                        r#"<bpmn:incoming>Flow_{}_{}"</bpmn:incoming>"#,
                        edge.from, edge.to
                    ));
                }

                bpmn.push_str(r#"</bpmn:endEvent>"#);
            }

            // Tasks and Activities
            BpmnEvent::ActivityTask(label)
            | BpmnEvent::ActivitySubprocess(label)
            | BpmnEvent::ActivityCallActivity(label)
            | BpmnEvent::ActivityEventSubprocess(label)
            | BpmnEvent::ActivityTransaction(label)
            | BpmnEvent::TaskUser(label)
            | BpmnEvent::TaskService(label)
            | BpmnEvent::TaskBusinessRule(label)
            | BpmnEvent::TaskScript(label) => {
                let element_type = match event {
                    BpmnEvent::ActivityTask(_) => "task",
                    BpmnEvent::TaskUser(_) => "userTask",
                    BpmnEvent::TaskService(_) => "serviceTask",
                    BpmnEvent::TaskBusinessRule(_) => "businessRuleTask",
                    BpmnEvent::TaskScript(_) => "scriptTask",
                    BpmnEvent::ActivitySubprocess(_) => "subProcess",
                    BpmnEvent::ActivityCallActivity(_) => "callActivity",
                    BpmnEvent::ActivityEventSubprocess(_) => "subProcess triggeredByEvent=\"true\"",
                    BpmnEvent::ActivityTransaction(_) => "transaction",
                    _ => "task",
                };

                bpmn.push_str(&format!(
                    r#"<bpmn:{} id="{}" name="{}">"#,
                    element_type,
                    get_node_bpmn_id(node),
                    label
                ));

                // Add incoming flows
                for edge in graph.edges.iter().filter(|e| e.to == node.id) {
                    bpmn.push_str(&format!(
                        r#"<bpmn:incoming>Flow_{}_{}"</bpmn:incoming>"#,
                        edge.from, edge.to
                    ));
                }

                // Add outgoing flows
                for edge in graph.edges.iter().filter(|e| e.from == node.id) {
                    bpmn.push_str(&format!(
                        r#"<bpmn:outgoing>Flow_{}_{}"</bpmn:outgoing>"#,
                        edge.from, edge.to
                    ));
                }

                bpmn.push_str(&format!(r#"</bpmn:{}>"#, element_type));
            }

            // Gateways
            BpmnEvent::GatewayExclusive | BpmnEvent::GatewayInclusive | BpmnEvent::GatewayJoin(_) => {
                let element_type = match event {
                    BpmnEvent::GatewayExclusive => "exclusiveGateway",
                    BpmnEvent::GatewayInclusive => "inclusiveGateway",
                    BpmnEvent::GatewayJoin(_) => "parallelGateway",
                    _ => "exclusiveGateway",
                };

                bpmn.push_str(&format!(
                    r#"<bpmn:{} id="{}">"#,
                    element_type,
                    get_node_bpmn_id(node),
                ));

                // Add incoming flows
                for edge in graph.edges.iter().filter(|e| e.to == node.id) {
                    bpmn.push_str(&format!(
                        r#"<bpmn:incoming>Flow_{}_{}"</bpmn:incoming>"#,
                        edge.from, edge.to
                    ));
                }

                // Add outgoing flows
                for edge in graph.edges.iter().filter(|e| e.from == node.id) {
                    bpmn.push_str(&format!(
                        r#"<bpmn:outgoing>Flow_{}_{}"</bpmn:outgoing>"#,
                        edge.from, edge.to
                    ));
                }

                bpmn.push_str(&format!(r#"</bpmn:{}>"#, element_type));
            }

            // Data Objects
            BpmnEvent::DataStoreReference(label) => {
                bpmn.push_str(&format!(
                    r#"<bpmn:dataStoreReference id="{}" name="{}" />"#,
                    get_node_bpmn_id(node),
                    label
                ));
            }
            BpmnEvent::DataObjectReference(label) => {
                bpmn.push_str(&format!(
                    r#"<bpmn:dataObjectReference id="{}" name="{}" />"#,
                    get_node_bpmn_id(node),
                    label
                ));
            }
            _ => {}
        }
    }
}

fn generate_sequence_flows(
    bpmn: &mut String,
    graph: &Graph,
    pool_nodes: &Vec<&Node>,
) {
    let node_ids: HashSet<usize> = pool_nodes.iter().map(|node| node.id).collect();

    for edge in &graph.edges {
        if node_ids.contains(&edge.from) && node_ids.contains(&edge.to) {
            let from_node = graph.get_node_by_id(edge.from).unwrap();
            let to_node = graph.get_node_by_id(edge.to).unwrap();

            let source_ref = get_node_bpmn_id(from_node);
            let target_ref = get_node_bpmn_id(to_node);

            // Lisa sequenceFlow element
            bpmn.push_str(&format!(
                r#"<bpmn:sequenceFlow id="Flow_{}_{}" sourceRef="{}" targetRef="{}" />"#,
                edge.from, edge.to, source_ref, target_ref
            ));

            // Lisa BPMNEdge element ja waypoints
            bpmn.push_str(&format!(
                r#"<bpmndi:BPMNEdge id="Flow_{}_{}_di" bpmnElement="Flow_{}_{}">"#,
                edge.from, edge.to, edge.from, edge.to
            ));

            if let Some(points) = &edge.adjusted_points {
                for (x, y) in points {
                    bpmn.push_str(&format!(
                        r#"<di:waypoint x="{:.2}" y="{:.2}" />"#,
                        x, y
                    ));
                }
            } else {
                // Kui adjusted_points puudub, lisa hoiatus
                eprintln!(
                    "Warning: Edge {} -> {} has no adjusted_points.",
                    edge.from, edge.to
                );
            }

            bpmn.push_str(r#"</bpmndi:BPMNEdge>"#);
        }
    }
}




fn get_node_bpmn_id(node: &Node) -> String {
    if let Some(event) = &node.event {
        match event {
            BpmnEvent::Start(_) | BpmnEvent::StartTimerEvent(_)
            | BpmnEvent::StartSignalEvent(_) | BpmnEvent::StartMessageEvent(_)
            | BpmnEvent::StartConditionalEvent(_) => format!("StartEvent_{}", node.id),

            BpmnEvent::End(_) | BpmnEvent::EndErrorEvent(_)
            | BpmnEvent::EndCancelEvent(_) | BpmnEvent::EndSignalEvent(_)
            | BpmnEvent::EndMessageEvent(_) | BpmnEvent::EndTerminateEvent(_)
            | BpmnEvent::EndEscalationEvent(_) | BpmnEvent::EndCompensationEvent(_) => {
                format!("EndEvent_{}", node.id)
            }

            BpmnEvent::ActivityTask(_)
            | BpmnEvent::TaskUser(_)
            | BpmnEvent::TaskService(_)
            | BpmnEvent::TaskBusinessRule(_)
            | BpmnEvent::TaskScript(_) => format!("Activity_{}", node.id),

            BpmnEvent::ActivitySubprocess(_) => format!("SubProcess_{}", node.id),
            BpmnEvent::ActivityCallActivity(_) => format!("CallActivity_{}", node.id),
            BpmnEvent::ActivityEventSubprocess(_) => format!("EventSubProcess_{}", node.id),
            BpmnEvent::ActivityTransaction(_) => format!("Transaction_{}", node.id),

            BpmnEvent::GatewayExclusive
            | BpmnEvent::GatewayInclusive
            | BpmnEvent::GatewayJoin(_) => format!("Gateway_{}", node.id),

            BpmnEvent::DataStoreReference(_) => format!("DataStoreReference_{}", node.id),
            BpmnEvent::DataObjectReference(_) => format!("DataObjectReference_{}", node.id),

            // Add other event types as needed
            _ => format!("Node_{}", node.id),
        }
    } else {
        format!("Node_{}", node.id)
    }
}

pub fn get_node_size(event: &BpmnEvent) -> (usize, usize) {
    match event {
        // Start Events
        BpmnEvent::Start(_)
        | BpmnEvent::StartTimerEvent(_)
        | BpmnEvent::StartSignalEvent(_)
        | BpmnEvent::StartMessageEvent(_)
        | BpmnEvent::StartConditionalEvent(_) => (36, 36),

        // End Events
        BpmnEvent::End(_)
        | BpmnEvent::EndErrorEvent(_)
        | BpmnEvent::EndCancelEvent(_)
        | BpmnEvent::EndSignalEvent(_)
        | BpmnEvent::EndMessageEvent(_)
        | BpmnEvent::EndTerminateEvent(_)
        | BpmnEvent::EndEscalationEvent(_)
        | BpmnEvent::EndCompensationEvent(_) => (36, 36),

        // Gateways
        BpmnEvent::GatewayExclusive
        | BpmnEvent::GatewayInclusive
        | BpmnEvent::GatewayJoin(_) => (50, 50),

        // Activities
        BpmnEvent::ActivityTask(_)
        | BpmnEvent::ActivityCallActivity(_)
        | BpmnEvent::TaskUser(_)
        | BpmnEvent::TaskService(_)
        | BpmnEvent::TaskBusinessRule(_)
        | BpmnEvent::TaskScript(_) => (100, 80),

        // Subprocesses and Transactions (expanded)
        BpmnEvent::ActivitySubprocess(_)
        | BpmnEvent::ActivityEventSubprocess(_)
        | BpmnEvent::ActivityTransaction(_) => (350, 200),

        // Data Objects
        BpmnEvent::DataStoreReference(_) => (50, 50),
        BpmnEvent::DataObjectReference(_) => (36, 50),

        // Default case
        _ => (100, 80),
    }
}

pub fn export_to_xml(bpmn: &String) {
    // Write BPMN to file
    let file_path = "generated_bpmn.bpmn";
    let mut file = File::create(file_path).expect("Unable to create file");
    file.write_all(bpmn.as_bytes())
        .expect("Unable to write data");

    println!("BPMN file generated at: {}", file_path);
}
