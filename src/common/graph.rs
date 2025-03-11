// graph.rs
use crate::common::bpmn_event::BpmnEvent;
use crate::common::dataedge::DataEdge;
use crate::common::datanode::DataNode;
use crate::common::edge::Edge;
use crate::common::node::Node;
use crate::common::pool::Pool;
use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use std::fmt;

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

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, text: Option<String>) -> EdgeId {
        let edge_id = EdgeId(self.edges.len());
        self.edges.push(Edge {
            from,
            to,
            text,
            bend_points: None, // Empty on creation, will be filled in assign_bend_points
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
            above: true,
            ..Default::default()
        });

        id
    }

    pub fn add_data_edge(&mut self, from: DataNodeId, to: NodeId, text: Option<String>) -> usize {
        let index = self.data_edges.len();
        self.data_edges.push(DataEdge {
            from,
            to,
            text,
            is_reversed: false,
            bend_points: None,
        });
        index
    }

    pub fn add_data_edge_reversed(
        &mut self,
        from: DataNodeId,
        to: NodeId,
        text: Option<String>,
    ) -> usize {
        let index = self.data_edges.len();
        self.data_edges.push(DataEdge {
            from,
            to,
            text,
            is_reversed: true,
            bend_points: None,
        });
        index
    }

    pub fn get_nodes_by_pool_name(&self, pool_name: Option<String>) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.pool == pool_name).collect()
    }

    pub fn sort_data_nodes(&mut self) {
        self.data_nodes.sort_by(|a, b| {
            let pool_a_pos = Self::get_pool_index(&self.pools, &a.pool);
            let pool_b_pos = Self::get_pool_index(&self.pools, &b.pool);

            let lane_a_pos = Self::get_lane_index(&self.pools, &a.pool, &a.lane);
            let lane_b_pos = Self::get_lane_index(&self.pools, &b.pool, &b.lane);

            let layer_a =
                a.layer_id.unwrap_or(usize::MAX) as f64 + if a.uses_half_layer { 0.5 } else { 0.0 };
            let layer_b =
                b.layer_id.unwrap_or(usize::MAX) as f64 + if b.uses_half_layer { 0.5 } else { 0.0 };

            pool_a_pos
                .cmp(&pool_b_pos)
                .then(lane_a_pos.cmp(&lane_b_pos))
                .then(layer_a.partial_cmp(&layer_b).unwrap_or(Equal))
        });
    }

    pub fn sort_data_nodes_by_average(&mut self, dn_averages: &HashMap<DataNodeId, f64>) {
        self.data_nodes.sort_by(|a, b| {
            let pool_a_pos = Self::get_pool_index(&self.pools, &a.pool);
            let pool_b_pos = Self::get_pool_index(&self.pools, &b.pool);

            let lane_a_pos = Self::get_lane_index(&self.pools, &a.pool, &a.lane);
            let lane_b_pos = Self::get_lane_index(&self.pools, &b.pool, &b.lane);

            let layer_a =
                a.layer_id.unwrap_or(usize::MAX) as f64 + if a.uses_half_layer { 0.5 } else { 0.0 };
            let layer_b =
                b.layer_id.unwrap_or(usize::MAX) as f64 + if b.uses_half_layer { 0.5 } else { 0.0 };

            let avg_a = dn_averages.get(&a.id).cloned().unwrap_or(0.0);
            let avg_b = dn_averages.get(&b.id).cloned().unwrap_or(0.0);

            pool_a_pos
                .cmp(&pool_b_pos)
                .then(lane_a_pos.cmp(&lane_b_pos))
                .then(layer_a.partial_cmp(&layer_b).unwrap_or(Equal))
                .then(avg_a.partial_cmp(&avg_b).unwrap_or(Equal))
        });
    }

    fn get_pool_index(pools: &Vec<Pool>, pool_id: &Option<String>) -> usize {
        pools
            .iter()
            .position(|p| &p.pool_name == pool_id)
            .unwrap_or(usize::MAX)
    }

    fn get_lane_index(
        pools: &Vec<Pool>,
        pool_id: &Option<String>,
        lane_id: &Option<String>,
    ) -> usize {
        for (_, pool) in pools.iter().enumerate() {
            if &pool.pool_name == pool_id {
                return pool
                    .lanes
                    .iter()
                    .position(|l| &l.lane == lane_id)
                    .unwrap_or(usize::MAX);
            }
        }
        usize::MAX
    }
}
