use crate::common::bpmn_event::get_node_size;
use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::EdgeId;
use crate::common::graph::Graph;
use crate::common::node::Node;
use crate::lexer::EventMeta;
use std::collections::HashMap;
use std::fmt::Display;

struct IncomingOutgoing<'a>(&'a [EdgeId], &'a [EdgeId]);

impl Display for IncomingOutgoing<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in self.0 {
            writeln!(f, "      <bpmn:incoming>Flow_{}</bpmn:incoming>", e.0)?;
        }
        for e in self.1 {
            writeln!(f, "      <bpmn:outgoing>Flow_{}</bpmn:outgoing>", e.0)?;
        }
        Ok(())
    }
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
"#,
    );

    let has_pools = !matches!(&graph.pools[..], [pool] if pool.pool_name.is_none() && matches!(&pool.lanes[..], [lane] if lane.lane.is_none()));

    if has_pools {
        bpmn.push_str("  <bpmn:collaboration id=\"Collaboration_1\">\n");

        for (id, pool) in graph.pools.iter().enumerate() {
            bpmn.push_str(&format!(
                "    <bpmn:participant id=\"Participant_{id}\" name=\"{}\" processRef=\"Process_{id}\" />\n", pool.pool_name.as_ref().map_or("", AsRef::as_ref)
            ));
        }
        bpmn.push_str("  </bpmn:collaboration>\n");
    }

    for (pool_id, pool) in graph.pools.iter().enumerate() {
        bpmn.push_str(&format!(
            "  <bpmn:process id=\"Process_{pool_id}\" isExecutable=\"true\">\n"
        ));
        if pool.lanes.len() > 1 {
            bpmn.push_str(&format!("    <bpmn:laneSet id=\"LaneSet_{}\">\n", pool_id));

            for (lane_id, lane) in pool.lanes.iter().enumerate() {
                bpmn.push_str(&format!(
                    "      <bpmn:lane id=\"Lane_{lane_id}\" name=\"{}\">\n",
                    lane.lane.clone().unwrap_or_default()
                ));

                for node_id in &lane.nodes {
                    bpmn.push_str(&format!(
                        "        <bpmn:flowNodeRef>Node_{node_id}</bpmn:flowNodeRef>\n"
                    ));
                }

                bpmn.push_str("      </bpmn:lane>\n");
            }

            bpmn.push_str("    </bpmn:laneSet>\n");
        }

        graph.nodes.iter().enumerate().for_each(|(node_id, node)| {
            if node.pool == pool.pool_name {
                write_process_node(&mut bpmn, node_id, node);
            }
        });

        // Generate sequence flows
        for (edge_id, edge) in graph.edges.iter().enumerate() {
            if graph.nodes[edge.from.0].pool == pool.pool_name
                && graph.nodes[edge.to.0].pool == pool.pool_name
            {
                bpmn.push_str(&format!(
                "    <bpmn:sequenceFlow id=\"Flow_{edge_id}\" sourceRef=\"Node_{}\" targetRef=\"Node_{}\" />\n",
                edge.from.0, edge.to.0
            ));
            }
        }

        bpmn.push_str("  </bpmn:process>\n");
    }

    // Add BPMN diagram elements (BPMNPlane and BPMNShape)
    if has_pools {
        bpmn.push_str(
            r#"  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Collaboration_1">
"#,
        );
    } else {
        bpmn.push_str(
            r#"  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_0">
"#,
        );
    }

    for (pool_id, pool) in graph.pools.iter().enumerate() {
        bpmn.push_str(&format!(
            r#"      <bpmndi:BPMNShape id="Participant_{pool_id}_di" bpmnElement="Participant_{pool_id}" isHorizontal="true" isExpanded="true">
        <dc:Bounds x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" />
      </bpmndi:BPMNShape>
"#,
            /* x */ pool.x.unwrap_or_default(),
            /* y */ pool.y.unwrap_or_default(),
            /* width */ pool.width.unwrap_or_default(),
            /* height */ pool.height.unwrap_or_default(),
        ));

        for (lane_id, lane) in pool.lanes.iter().enumerate() {
            bpmn.push_str(&format!(
                r#"      <bpmndi:BPMNShape id="Lane_{lane_id}_di" bpmnElement="Lane_{lane_id}" isHorizontal="true">
        <dc:Bounds x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" />
      </bpmndi:BPMNShape>
"#,
                /* x */ lane.x.unwrap_or_default(),
                /* y */ lane.y.unwrap_or_default(),
                /* width */ lane.width.unwrap_or_default(),
                /* height */ lane.height.unwrap_or_default(),
            ));
        }
    }

    graph.nodes.iter().enumerate().for_each(|(node_id, node)| {
        let (width, height) = if let Some(event) = &node.event {
            get_node_size(event)
        } else {
            (100, 80) // Default size if event is None
        };

        bpmn.push_str(&format!(
            r#"      <bpmndi:BPMNShape id="Node_{node_id}_di" bpmnElement="Node_{node_id}" {}>
        <dc:Bounds x="{:.2}" y="{:.2}" width="{}" height="{}" />
      </bpmndi:BPMNShape>
"#,
            AdditionalShapeInfo(node),
            node.x.unwrap_or_default() + node.x_offset.unwrap_or_default(),
            node.y.unwrap_or_default() + node.y_offset.unwrap_or_default(),
            width,
            height,
        ));
    });

    // Add BPMNEdge elements
    for (edge_id, edge) in graph.edges.iter().enumerate() {
        bpmn.push_str(&format!(
            "      <bpmndi:BPMNEdge id=\"Flow_{edge_id}_di\" bpmnElement=\"Flow_{edge_id}\">\n"
        ));

        edge.bend_points.iter().flatten().for_each(|(x, y)| {
            bpmn.push_str(&format!(
                "        <di:waypoint x=\"{x:.2}\" y=\"{y:.2}\" />\n"
            ))
        });

        bpmn.push_str("      </bpmndi:BPMNEdge>\n");
    }

    bpmn.push_str(
        r#"    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#,
    );

    bpmn
}

fn write_process_node(bpmn: &mut String, node_id: usize, node: &Node) {
    let incomingoutgoing = IncomingOutgoing(&node.incoming, &node.outgoing);
    if let Some(event) = &node.event {
        match event {
            // Gateways
            BpmnEvent::Gateway(gt) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:{0}Gateway id="Node_{node_id}">
{incomingoutgoing}
    </bpmn:{0}Gateway>
"#,
                    match gt {
                        crate::lexer::GatewayType::Exclusive => "exclusive",
                        crate::lexer::GatewayType::Parallel => "parallel",
                        crate::lexer::GatewayType::Inclusive => "inclusive",
                        crate::lexer::GatewayType::Event => "eventBased",
                    }
                ));
            }

            // Activities
            | BpmnEvent::ActivityTask(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:task id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:task>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::ActivitySubprocess(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:subProcess id="Node_{node_id}" name="{}" triggeredByEvent="false">
{incomingoutgoing}
    </bpmn:subProcess>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::ActivityCallActivity(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:callActivity id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:callActivity>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::ActivityEventSubprocess(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:subProcess id="Node_{node_id}" name="{}" triggeredByEvent="true">
{incomingoutgoing}
    </bpmn:subProcess>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::ActivityTransaction(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:transaction id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:transaction>
"#,
                    meta.display_text
                ));
            }

            // Start Events
            BpmnEvent::Start(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:startEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:startEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::StartTimerEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:startEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:timerEventDefinition />
    </bpmn:startEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::StartSignalEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:startEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:signalEventDefinition />
    </bpmn:startEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::StartMessageEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:startEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:messageEventDefinition />
    </bpmn:startEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::StartConditionalEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:startEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:conditionalEventDefinition>
      <bpmn:condition xsi:type="bpmn:tFormalExpression">/* Your condition here */</bpmn:condition>
    </bpmn:conditionalEventDefinition>
    </bpmn:startEvent>
"#,
                    meta.node_meta.display_text
                ));
            }

            // End Events
            BpmnEvent::Middle(EventMeta {
                node_meta: meta, ..
            }) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:intermediateThrowEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:intermediateThrowEvent>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::End(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndErrorEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:errorEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndCancelEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:cancelEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndSignalEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:signalEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndMessageEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:messageEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndTerminateEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:terminateEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndEscalationEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:escalationEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }
            BpmnEvent::EndCompensationEvent(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:endEvent id="Node_{node_id}" name="{}">
{incomingoutgoing}
    <bpmn:compensateEventDefinition />
    </bpmn:endEvent>
"#,
                    meta.node_meta.display_text
                ));
            }

            // Boundary Events
            BpmnEvent::BoundaryEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundaryErrorEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    <bpmn:errorEventDefinition />
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundaryTimerEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    <bpmn:timerEventDefinition />
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundarySignalEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    <bpmn:signalEventDefinition />
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundaryMessageEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    <bpmn:messageEventDefinition />
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundaryEscalationEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    <bpmn:escalationEventDefinition />
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundaryConditionalEvent(meta, attached_to, cancel_activity) => {
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="{}">
{incomingoutgoing}
    <bpmn:conditionalEventDefinition>
      <bpmn:condition xsi:type="bpmn:tFormalExpression">/* Your condition here */</bpmn:condition>
    </bpmn:conditionalEventDefinition>
    </bpmn:boundaryEvent>
"#,
                        meta,
                        if *cancel_activity { "true" } else { "false" },
                    ));
            }
            BpmnEvent::BoundaryCompensationEvent(meta, attached_to) => {
                // Compensation boundary events are always non-interrupting
                bpmn.push_str(&format!(
                        r#"    <bpmn:boundaryEvent id="Node_{node_id}" name="{}" attachedToRef="Node_{attached_to}" cancelActivity="false">
{incomingoutgoing}
    <bpmn:compensateEventDefinition />
    </bpmn:boundaryEvent>
"#,
                        meta
                    ));
            }

            // Data Objects
            BpmnEvent::DataStoreReference(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:dataStoreReference id="Node_{node_id}" name="{}" />
"#,
                    meta
                ));
            }
            BpmnEvent::DataObjectReference(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:dataObjectReference id="Node_{node_id}" name="{}" />
"#,
                    meta
                ));
            }

            // Tasks
            BpmnEvent::TaskUser(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:userTask id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:userTask>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::TaskService(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:serviceTask id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:serviceTask>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::TaskBusinessRule(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:businessRuleTask id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:businessRuleTask>
"#,
                    meta.display_text
                ));
            }
            BpmnEvent::TaskScript(meta) => {
                bpmn.push_str(&format!(
                    r#"    <bpmn:scriptTask id="Node_{node_id}" name="{}">
{incomingoutgoing}
    </bpmn:scriptTask>
"#,
                    meta.display_text
                ));
            }

            // Default case
            _ => {}
        }
    }
}

struct AdditionalShapeInfo<'a>(&'a Node);

impl Display for AdditionalShapeInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.event {
            Some(BpmnEvent::Gateway(_)) => write!(f, " isMarkerVisible=\"true\""),
            _ => Ok(()),
        }
    }
}
