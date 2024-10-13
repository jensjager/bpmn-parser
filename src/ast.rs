    // src/ast.rs

    #[derive(Debug, Clone)]
    pub struct BpmnGraph {
        pub nodes: Vec<BpmnNode>,  // List of nodes in the graph
        pub edges: Vec<BpmnEdge>,  // List of edges connecting the nodes
    }

    #[derive(Debug, Clone)]
    pub struct BpmnNode {
        pub id: usize,            // Unique identifier for each node
        pub event: BpmnEvent,     // Event associated with the node
    }

    #[derive(Debug, Clone)]
    pub struct BpmnEdge {
        pub from: usize,          // ID of the source node
        pub to: usize,            // ID of the destination node
    }

    #[derive(Debug, Clone)]
    pub enum BpmnEvent {
        Start(String),  // Start event with label
        Middle(String), // Middle event with label
        End(String),    // End event with label
    }

    impl BpmnGraph {
        // Creates a new, empty BPMN graph
        pub fn new() -> Self {
            BpmnGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
            }
        }

        // Adds a new node to the graph and returns its ID
        pub fn add_node(&mut self, event: BpmnEvent) -> usize {
            let id = self.nodes.len(); // Unique ID based on the current length of the nodes list
            self.nodes.push(BpmnNode { id, event });
            id
        }

        // Adds a directed edge between two nodes
        pub fn add_edge(&mut self, from: usize, to: usize) {
            self.edges.push(BpmnEdge { from, to });
        }
    }

    impl BpmnGraph {
        // Helper function to print the structure of the graph (nodes and edges)
        pub fn print_graph(&self) {
            println!("Nodes:");
            for node in &self.nodes {
                match &node.event {
                    BpmnEvent::Start(label) => println!("  Start Event: {} (ID: {})", label, node.id),
                    BpmnEvent::Middle(label) => println!("  Middle Event: {} (ID: {})", label, node.id),
                    BpmnEvent::End(label) => println!("  End Event: {} (ID: {})", label, node.id),
                }
            }

            println!("Edges:");
            for edge in &self.edges {
                println!("  From Node {} to Node {}", edge.from, edge.to);
            }
        }
    }
