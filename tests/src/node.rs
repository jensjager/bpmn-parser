pub enum NodeType {
    StartEvent,
    IntermediateEvent,
    EndEvent,
}

pub struct Node {
    pub id: usize,
    pub node_type: NodeType,
    pub layer: Option<i32>,
}

impl Node {
    pub fn new(id: usize, node_type: NodeType, layer: Option<i32>) -> Node {
        Node { id, node_type, layer }
    }
}
