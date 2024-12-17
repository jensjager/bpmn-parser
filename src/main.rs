// src/main.rs

mod common;
mod layout;
mod lexer;
mod parser;
mod read_input;
mod test;
mod to_xml;
use crate::read_input::read_lines;
use crate::to_xml::generate_bpmn;
use common::bpmn_event::BpmnEvent;
use layout::assign_bend_points::assign_bend_points;
use layout::crossing_minimization::reduce_crossings;
use layout::node_positioning::assign_xy_to_nodes;
use layout::solve_layer_assignment::solve_layer_assignment;
use lexer::Lexer;
use parser::Parser;

use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Use "input.txt" if no argument is provided
    let input_data = if args.len() < 2 {
        "input.txt".to_string()
    } else {
        args[1].clone()
    };

    let output_data = if args.len() < 3 {
        "".to_string()
    } else {
        args[2].clone()
    };

    // Check if the output format is valid
    if output_data != "pdf" && output_data != "svg" && output_data != "png" && output_data != "" {
        eprintln!("Error: Output format must be pdf, svg, png or left blank for xml");
        std::process::exit(1);
    }

    // Try to read the lines from the input file or exit if there's an error
    let input = match read_lines(&input_data) {
        Ok(lines) => lines,
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_data, e);
            std::process::exit(1);
        }
    };

    let bpmn = run_parser(&input);

    to_xml::export_to_xml(&bpmn);

    if !output_data.is_empty() {
        match convert_bpmn_to_image(output_data) {
            Ok(_) => println!("Successfully converted BPMN to image"),
            Err(e) => eprintln!("{}", e),
        }
    }
}

pub fn run_parser(input: &str) -> String {
    // Initialize the lexer with the input
    let lexer = Lexer::new(input);

    // Initialize the parser with the lexer
    let mut parser = Parser::new(lexer);

    // Parse the input and handle the result
    match parser.parse() {
        Ok(mut graph) => {
            println!("Parsed BPMN Graph:");

            graph.print_graph();
            solve_layer_assignment(&mut graph);
            // reduce_crossings(&mut graph, &layers);
            assign_xy_to_nodes(&mut graph);
            assign_bend_points(&mut graph);

            // for node in &graph.nodes {
            //     if let Some(event) = &node.event {
            //         match event {
            //             BpmnEvent::Start(label) => {
            //                 println!("  Start Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::Middle(label) => {
            //                 println!("  Middle Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::End(label) => {
            //                 println!("  End Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::GatewayExclusive => {
            //                 println!("  Gateway Exclusive Event (ID: {})", node.id);
            //             }
            //             BpmnEvent::GatewayJoin(label) => {
            //                 println!("  Gateway Join Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::ActivityTask(label) => {
            //                 println!("  Activity Task: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::GatewayInclusive => {
            //                 println!("  Gateway Inclusive Event (ID: {})", node.id);
            //             }

            //             BpmnEvent::ActivitySubprocess(label) => {
            //                 println!("  Activity Subprocess: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::ActivityCallActivity(label) => {
            //                 println!("  Activity Call Activity: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::ActivityEventSubprocess(label) => {
            //                 println!("  Activity Event Subprocess: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::ActivityTransaction(label) => {
            //                 println!("  Activity Transaction: {} (ID: {})", label, node.id);
            //             }

            //             BpmnEvent::StartTimerEvent(label) => {
            //                 println!("  Start Timer Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::StartSignalEvent(label) => {
            //                 println!("  Start Signal Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::StartMessageEvent(label) => {
            //                 println!("  Start Message Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::StartConditionalEvent(label) => {
            //                 println!("  Start Conditional Event: {} (ID: {})", label, node.id);
            //             }

            //             BpmnEvent::EndErrorEvent(label) => {
            //                 println!("  End Error Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::EndCancelEvent(label) => {
            //                 println!("  End Cancel Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::EndSignalEvent(label) => {
            //                 println!("  End Signal Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::EndMessageEvent(label) => {
            //                 println!("  End Message Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::EndTerminateEvent(label) => {
            //                 println!("  End Terminate Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::EndEscalationEvent(label) => {
            //                 println!("  End Escalation Event: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::EndCompensationEvent(label) => {
            //                 println!("  End Compensation Event: {} (ID: {})", label, node.id);
            //             }

            //             BpmnEvent::BoundaryEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryErrorEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Error Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryTimerEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Timer Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryCancelEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Cancel Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundarySignalEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Signal Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryMessageEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Message Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryEscalationEvent(label, attached_to, cancel_activity) => {
            //                 println!(
            //                     "  Boundary Escalation Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryConditionalEvent(
            //                 label,
            //                 attached_to,
            //                 cancel_activity,
            //             ) => {
            //                 println!(
            //                     "  Boundary Conditional Event: {} (ID: {}), attached to {}, cancel_activity: {}",
            //                     label, node.id, attached_to, cancel_activity
            //                 );
            //             }
            //             BpmnEvent::BoundaryCompensationEvent(label, attached_to) => {
            //                 println!(
            //                     "  Boundary Compensation Event: {} (ID: {}), attached to {}",
            //                     label, node.id, attached_to
            //                 );
            //             }

            //             BpmnEvent::DataStoreReference(label) => {
            //                 println!("  Data Store Reference: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::DataObjectReference(label) => {
            //                 println!("  Data Object Reference: {} (ID: {})", label, node.id);
            //             }

            //             BpmnEvent::TaskUser(label) => {
            //                 println!("  User Task: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::TaskService(label) => {
            //                 println!("  Service Task: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::TaskBusinessRule(label) => {
            //                 println!("  Business Rule Task: {} (ID: {})", label, node.id);
            //             }
            //             BpmnEvent::TaskScript(label) => {
            //                 println!("  Script Task: {} (ID: {})", label, node.id);
            //             }
            //         }
            //     } else {
            //         println!("  No Event (ID: {})", node.id);
            //     }
            // }

            println!("Edges:");
            for edge in &graph.edges {
                if let Some(text) = &edge.text {
                    println!("  From Node {} to Node {}: '{}'", edge.from, edge.to, text);
                } else {
                    println!("  From Node {} to Node {}", edge.from, edge.to);
                }
            }
            return generate_bpmn(&graph);
        }
        Err(e) => {
            eprintln!("Failed to initialize parser:\n{}", e);
            e.to_string()
        },
    }

}

fn convert_bpmn_to_image(output_type: String) -> Result<(), String> {
    let args = format!("generated_bpmn.bpmn:{}", output_type);

    let status = Command::new("bpmn-to-image").arg(args).output();

    match status {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!(
                    "Error: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}
