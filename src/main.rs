#![feature(let_chains)]
#![feature(map_try_insert)]
#![feature(never_type)]

mod common;
mod layout;
mod lexer;
mod parser;
mod to_xml;
use clap::Parser;
use common::bpmn_event::BpmnEvent;
use layout::all_crossing_minimization::reduce_all_crossings;
use layout::assign_bend_points::assign_bend_points;
use layout::dummy_node_generation::generate_dummy_nodes;
use layout::solve_data_layer_assignment::solve_data_layer_assignment;
use layout::solve_layer_assignment::solve_layer_assignment;
use layout::xy_ilp::assign_xy_ilp;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input DSL file. If missing, the input file is read from STDIN.
    #[arg(short, long, value_name = "IN_FILE")]
    input: Option<std::path::PathBuf>,

    /// Output XML file. If missing, the xml data will be written to STDOUT.
    #[arg(short, long, value_name = "OUT_FILE")]
    output: Option<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let bpmd = cli.input.map_or_else(
        || std::io::read_to_string(std::io::stdin()),
        std::fs::read_to_string,
    )?;

    let bpmn = bpmd_to_bpmn(bpmd)?;

    match cli.output {
        Some(pb) => std::fs::write(pb, bpmn)?,
        None => print!("{bpmn}"),
    };

    Ok(())
}

pub fn bpmd_to_bpmn(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut graph = parser::parse(input)?;
    solve_layer_assignment(&mut graph);
    generate_dummy_nodes(&mut graph);
    graph.data_edges.iter().for_each(|edge| {
        println!("{} - {}", edge.from, edge.to);
    });
    solve_data_layer_assignment(&mut graph);
    reduce_all_crossings(&mut graph);
    println!("Hello");
    assign_xy_ilp(&mut graph);
    assign_bend_points(&mut graph);
    for node in graph.nodes.iter() {
        if node.event.clone().unwrap() == BpmnEvent::Dummy() {
            dbg!(node);
        }
    }
    for edge in graph.edges.iter() {
        if edge.is_dummy {
            dbg!(edge);
        }
    }
    Ok(to_xml::generate_bpmn(&graph))
}
