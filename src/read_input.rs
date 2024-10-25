use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn read_lines<P>(filename: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let buf_reader = io::BufReader::new(file);
    
    let lines: Vec<String> = buf_reader.lines().collect::<Result<Vec<_>, _>>()?;
    Ok(lines.join("\n"))
}