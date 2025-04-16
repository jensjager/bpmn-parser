// graph.rs
use crate::common::bpmn_event::BpmnEvent;
use crate::common::dataedge::DataEdge;
use crate::common::datanode::DataNode;
use crate::common::dummy::Dummy;
use crate::common::edge::Edge;
use crate::common::node::Node;
use crate::common::pool::Pool;
use std::fmt::{self};

/// Represents a graph consisting of nodes and edges.
#[derive(Default)]
pub struct Graph {
    /// Nodes shall only be added to this Vec, but the order shall not be modified.
    /// Otherwise NodeIds will point to the wrong nodes.
    pub nodes: Vec<Node>,
    /// Edges shall only be added to this Vec, but the order shall not be modified.
    /// Otherwise EdgeIds will point to the wrong edges.
    pub edges: Vec<Edge>,
    pub pools: Vec<Pool>,
    pub data_nodes: Vec<DataNode>,
    pub data_edges: Vec<DataEdge>,
}

/// A Newtype to make sure that code outside of the module does not modify its value.
/// The invariant is that every created NodeId does point to some existing node.
#[derive(PartialEq, Default, Clone, Debug, Copy, Hash, Eq)]
pub struct NodeId(pub usize);

#[derive(PartialEq, Default, Clone, Debug, Copy, Hash, Eq)]
pub struct DataNodeId(pub usize);

#[derive(PartialEq, Default, Clone, Debug, Copy, Hash, Eq)]
pub struct EdgeId(pub usize);

#[derive(PartialEq, Default, Clone, Debug, Copy, Hash, Eq)]
pub struct DataEdgeId(pub usize);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for DataNodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Graph {
    /// Returns the node_id parameter for convenience on the caller side.
    pub fn add_node(
        &mut self,
        bpmn_event: BpmnEvent,
        pool: Option<String>,
        lane: Option<String>,
    ) -> NodeId {
        let node_id = NodeId(self.nodes.len());

        // Add node ID to the pool and lane stuff
        if let Some(pool) = self.pools.iter_mut().find(|l| l.pool_name == pool) {
            pool.add_node(lane.clone(), node_id);
        } else {
            let mut new_pool = Pool::new(pool.clone());
            new_pool.add_node(lane.clone(), node_id);
            self.pools.push(new_pool);
        }

        self.nodes.push(Node {
            id: node_id,
            event: Some(bpmn_event),
            pool,
            lane,
            ..Default::default()
        });

        node_id
    }

    pub fn add_dummy_node(
        &mut self,
        pool: Option<String>,
        lane: Option<String>,
        layer_id: Option<usize>,
    ) -> NodeId {
        let node_id = NodeId(self.nodes.len());

        if let Some(pool) = self.pools.iter_mut().find(|l| l.pool_name == pool) {
            pool.add_node(lane.clone(), node_id);
        } else {
            // TODO throw error
        }

        self.nodes.push(Node {
            id: node_id,
            event: Some(BpmnEvent::Dummy()),
            pool,
            lane,
            layer_id,
            ..Default::default()
        });

        node_id
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, text: Option<String>) -> EdgeId {
        let edge_id = EdgeId(self.edges.len());
        self.edges.push(Edge {
            from,
            to,
            text,
            bend_points: None, // Empty on creation, will be filled in assign_bend_points
            ..Default::default()
        });

        self.nodes[from.0].outgoing.push(edge_id);
        self.nodes[to.0].incoming.push(edge_id);
        edge_id
    }

    pub fn add_dummy_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        real_from: Option<usize>,
        real_to: Option<usize>,
        data_node: bool,
    ) -> EdgeId {
        let edge_id = EdgeId(self.edges.len());
        self.edges.push(Edge {
            from,
            to,
            bend_points: None, // Empty on creation, will be filled in assign_bend_points
            dummy: Some(Dummy {
                is_data: data_node,
                is_reversed: false,
                from: real_from,
                to: real_to,
                ..Default::default()
            }),
            ..Default::default()
        });

        self.nodes[from.0].outgoing.push(edge_id);
        self.nodes[to.0].incoming.push(edge_id);
        edge_id
    }

    pub fn add_dummy_edge_from_data_node(
        &mut self,
        from: NodeId,
        to: NodeId,
        real_from: usize,
        real_to: usize,
    ) -> EdgeId {
        let edge_id = EdgeId(self.edges.len());
        self.edges.push(Edge {
            from,
            to,
            bend_points: None, // Empty on creation, will be filled in assign_bend_points
            dummy: Some(Dummy {
                is_data: true,
                is_reversed: false,
                from: Some(real_from),
                to: Some(real_to),
                ..Default::default()
            }),
            ..Default::default()
        });

        self.nodes[from.0].outgoing.push(edge_id);
        self.nodes[to.0].incoming.push(edge_id);
        edge_id
    }

    pub fn add_data_node(
        &mut self,
        datatype: Option<BpmnEvent>,
        pool: Option<String>,
        lane: Option<String>,
    ) -> DataNodeId {
        let id = DataNodeId(self.data_nodes.len());

        self.data_nodes.push(DataNode {
            id,
            datatype,
            pool,
            lane,
            ..Default::default()
        });

        id
    }

    pub fn add_data_edge(&mut self, from: DataNodeId, to: NodeId, text: Option<String>) -> usize {
        let edge_id = self.data_edges.len();
        self.data_edges.push(DataEdge {
            from,
            to,
            text,
            is_reversed: false,
            bend_points: None,
            ..Default::default()
        });

        self.data_nodes[from.0].outgoing.push(DataEdgeId(edge_id));
        edge_id
    }

    pub fn add_data_edge_reversed(
        &mut self,
        from: DataNodeId,
        to: NodeId,
        text: Option<String>,
    ) -> usize {
        let edge_id = self.data_edges.len();
        self.data_edges.push(DataEdge {
            from,
            to,
            text,
            is_reversed: true,
            bend_points: None,
            ..Default::default()
        });

        self.data_nodes[from.0].incoming.push(DataEdgeId(edge_id));
        edge_id
    }

    pub fn add_dummy_data_edge(
        &mut self,
        from: DataNodeId,
        to: NodeId,
        real_from: usize,
        real_to: usize,
    ) -> usize {
        let edge_id = self.data_edges.len();
        self.data_edges.push(DataEdge {
            from,
            to,
            is_reversed: false,
            bend_points: None,
            dummy: Some(Dummy {
                is_data: true,
                is_reversed: false,
                from: Some(real_from),
                to: Some(real_to),
                ..Default::default()
            }),
            ..Default::default()
        });

        self.data_nodes[from.0].outgoing.push(DataEdgeId(edge_id));
        edge_id
    }

    pub fn add_dummy_data_edge_reversed(
        &mut self,
        from: DataNodeId,
        to: NodeId,
        real_from: usize,
        real_to: usize,
    ) -> usize {
        let edge_id = self.data_edges.len();
        self.data_edges.push(DataEdge {
            from,
            to,
            is_reversed: true,
            bend_points: None,
            dummy: Some(Dummy {
                is_data: true,
                is_reversed: true,
                from: Some(real_to),
                to: Some(real_from),
                ..Default::default()
            }),
            ..Default::default()
        });

        self.data_nodes[from.0].incoming.push(DataEdgeId(edge_id));
        edge_id
    }
}
