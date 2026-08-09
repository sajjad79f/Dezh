use std::collections::HashMap;
use std::sync::RwLock;

use contracts::CoreService;
use uuid::Uuid;

use crate::models::{Edge, Node};

/// Knowledge Graph -- stores relationships between assets (or anything
/// else) as a generic property graph. Nodes are typically DAI asset ids,
/// but DKG does not require that: it stores whatever id it is given.
pub struct DkgService {
    nodes: RwLock<HashMap<Uuid, Node>>,
    edges: RwLock<HashMap<Uuid, Edge>>,
}

impl DkgService {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            edges: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_node(
        &self,
        id: Uuid,
        label: impl Into<String>,
    ) {
        let node = Node {
            id,
            label: label.into(),
            properties: HashMap::new(),
        };

        self.nodes
            .write()
            .expect("DKG node store lock poisoned")
            .insert(id, node);
    }

    pub fn add_edge(
        &self,
        from: Uuid,
        to: Uuid,
        relation: impl Into<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();

        let edge = Edge {
            id,
            from,
            to,
            relation: relation.into(),
            properties: HashMap::new(),
        };

        self.edges
            .write()
            .expect("DKG edge store lock poisoned")
            .insert(id, edge);

        id
    }

    pub fn get_node(&self, id: Uuid) -> Option<Node> {
        self.nodes
            .read()
            .expect("DKG node store lock poisoned")
            .get(&id)
            .cloned()
    }

    pub fn list_nodes(&self) -> Vec<Node> {
        self.nodes
            .read()
            .expect("DKG node store lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn list_edges(&self) -> Vec<Edge> {
        self.edges
            .read()
            .expect("DKG edge store lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    /// Every outgoing edge from `id`, paired with the node it points to.
    pub fn neighbors(&self, id: Uuid) -> Vec<(Edge, Option<Node>)> {
        let edges = self.edges.read().expect("DKG edge store lock poisoned");
        let nodes = self.nodes.read().expect("DKG node store lock poisoned");

        edges
            .values()
            .filter(|edge| edge.from == id)
            .map(|edge| (edge.clone(), nodes.get(&edge.to).cloned()))
            .collect()
    }
}

impl CoreService for DkgService {
    fn name(&self) -> &'static str {
        "DKG"
    }

    fn initialize(&self) {
        println!("[DKG] initialized");
    }

    fn shutdown(&self) {
        println!("[DKG] shutdown");
    }
}
