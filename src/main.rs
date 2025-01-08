#![feature(let_chains)]
#![feature(map_try_insert)]
#![feature(never_type)]

mod common;
mod layout;
mod lexer;
mod parser;
mod to_xml;
use clap::Parser;
use layout::assign_bend_points::assign_bend_points;
use layout::crossing_minimization::reduce_crossings;
use layout::node_positioning::assign_xy_to_nodes;
use layout::solve_layer_assignment::solve_layer_assignment;

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
    reduce_crossings(&mut graph);
    assign_xy_to_nodes(&mut graph);
    assign_bend_points(&mut graph);

    Ok(to_xml::generate_bpmn(&graph))
}
