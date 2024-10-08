pub struct node {
    pub id: usize,
    pub layer: Option<i32>
}

impl node {
    pub fn new(id: usize, layer: Option<i32>) -> node {
        node { id, layer: None}
    }
}
