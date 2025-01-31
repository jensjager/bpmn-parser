use crate::common::graph::DataNodeId;
use crate::common::graph::NodeId;

#[derive(Debug, Clone)]
pub struct DataEdge {
    pub from: DataNodeId,
    pub to: NodeId,
    pub text: Option<String>,
    pub is_reversed: bool,
    pub bend_points: Option<Vec<(f64, f64)>>,
}

impl DataEdge {}
