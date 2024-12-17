// graph.rs
use crate::common::bpmn_event::BpmnEvent;
use crate::common::edge::Edge;
use crate::common::node::Node;
use crate::common::pool::Pool;

/// Represents a graph consisting of nodes and edges.
#[derive(Clone)]
pub struct Graph {
    pub pools: Vec<Pool>,    // Pools
    pub edges: Vec<Edge>,    // Edges
    pub last_node_id: usize, // Last used node ID
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            pools: Vec::new(),
            edges: Vec::new(),
            last_node_id: 0,
        }
    }

    pub fn add_node(
        &mut self,
        bpmn_event: BpmnEvent,
        id: Option<usize>,
        pool_node: Option<String>,
        lane_node: Option<String>,
    ) -> usize {
        // Generate or use provided node ID
        let node_id = id.unwrap_or_else(|| self.next_node_id());


        // Create new node
        let node = Node::new(
            node_id,
            None,
            None,
            Some(bpmn_event),
            pool_node.clone(),
            lane_node.clone(),
        );


        // Add node to appropriate pool
        let pool_name = pool_node.unwrap_or_default();
        if let Some(pool) = self
            .pools
            .iter_mut()
            .find(|p| p.get_pool_name() == pool_name)
        {
            pool.add_node(node);
        } else {
            let mut new_pool = Pool::new(pool_name);
            new_pool.add_node(node);
            self.pools.push(new_pool);
        }

        node_id
    }

    pub fn get_pools(&self) -> &Vec<Pool> {
        &self.pools
    }

    pub fn get_pools_mut(&mut self) -> &mut Vec<Pool> {
        &mut self.pools
    }

    pub fn take_edges(&mut self) -> Vec<Edge> {
        std::mem::take(&mut self.edges)
    }

    pub fn set_edges(&mut self, edges: Vec<Edge>) {
        self.edges = edges;
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    // Get the next node ID.
    pub fn next_node_id(&mut self) -> usize {
        self.last_node_id += 1; // Increment the last used ID
        self.last_node_id // Return the new ID
    }

    pub fn get_node_by_id(&self, id: usize) -> Option<&Node> {
        for pool in &self.pools {
            for lane in pool.get_lanes() {
                for node in lane.get_layers() {
                    if node.id == id {
                        return Some(node);
                    }
                }
            }
        }
        println!("Node with id {} not found", id);
        None
    }

    pub fn get_nodes_by_pool_name(&self, pool_name: &str) -> Vec<&Node> {
        self.pools
            .iter()
            .flat_map(|pool| pool.get_lanes())
            .flat_map(|lane| lane.get_layers())
            .filter(|node| node.pool.as_deref() == Some(pool_name))
            .collect()
    }

    pub fn print_graph(&self) {
        println!("Printing Graph");
        for pool in &self.pools {
            println!("Pool: {}", pool.get_pool_name());
            for lane in pool.get_lanes() {
                println!("  Lane: {}", lane.get_lane());
                for node in lane.get_layers() {
                    println!(
                        "    Node: {}, x: {}, y: {}, y_offset: {}",
                        node.id,
                        node.x.unwrap_or(0.0),
                        node.y.unwrap_or(0.0),
                        node.y_offset.unwrap_or(0.0)
                    );
                }
            }
        }
        println!("Printing edges");
        for edge in &self.edges {
            println!("  Edge: {} -> {}", edge.from, edge.to);
        }
    }
}
