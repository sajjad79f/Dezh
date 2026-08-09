use std::sync::Arc;
use uuid::Uuid;

use contracts::CoreService;

use crate::error::DkgResult;
use crate::models::{Edge, Node};
use crate::registry::GraphRegistry;

pub struct DkgService {
    registry: Arc<GraphRegistry>,
}

impl DkgService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(GraphRegistry::new()),
        }
    }

    pub fn add_node(&self, id: Uuid, label: impl Into<String>) -> DkgResult<()> {
        self.registry.add_node(id, label)
    }

    pub fn add_edge(
        &self,
        from: Uuid,
        to: Uuid,
        relation: impl Into<String>,
    ) -> DkgResult<Uuid> {
        self.registry.add_edge(from, to, relation)
    }

    pub fn get_node(&self, id: Uuid) -> Option<Node> {
        self.registry.get_node(id)
    }

    pub fn list_nodes(&self) -> Vec<Node> {
        self.registry.list_nodes()
    }

    pub fn list_edges(&self) -> Vec<Edge> {
        self.registry.list_edges()
    }

    pub fn neighbors(&self, id: Uuid) -> Vec<(Edge, Option<Node>)> {
        self.registry.neighbors(id)
    }

    pub fn remove_node(&self, id: Uuid) -> DkgResult<()> {
        self.registry.remove_node(id)
    }

    pub fn remove_edge(&self, id: Uuid) -> DkgResult<()> {
        self.registry.remove_edge(id)
    }

    pub fn node_count(&self) -> usize {
        self.registry.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.registry.edge_count()
    }
}

impl Clone for DkgService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
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