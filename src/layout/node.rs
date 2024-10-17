#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub layer: Option<i32>,
}

impl Node {
    pub fn new(id: usize, layer: Option<i32>) -> Self {
        Node { id, layer }
    }
}
