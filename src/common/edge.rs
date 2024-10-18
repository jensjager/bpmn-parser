//  edge.rs
#[derive(Debug)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub text: Option<String>
}

impl Edge {
    pub fn new(from: usize, to: usize, text: Option<String>) -> Self {
        Edge { from, to, text }
    }

    pub fn with_default_text(from: usize, to: usize) -> Self {
        Edge { from, to, text: None}
    }
}
