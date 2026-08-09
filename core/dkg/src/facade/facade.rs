use std::sync::Arc;
use uuid::Uuid;

use crate::error::DkgResult;
use crate::models::{Edge, Node};
use crate::service::DkgService;

pub struct DkgFacade {
    service: Arc<DkgService>,
}

impl DkgFacade {
    pub fn new() -> Self {
        Self {
            service: Arc::new(DkgService::new()),
        }
    }

    pub fn from_service(service: Arc<DkgService>) -> Self {
        Self { service }
    }

    pub fn add_node(&self, id: Uuid, label: impl Into<String>) -> DkgResult<()> {
        self.service.add_node(id, label)
    }

    pub fn add_edge(
        &self,
        from: Uuid,
        to: Uuid,
        relation: impl Into<String>,
    ) -> DkgResult<Uuid> {
        self.service.add_edge(from, to, relation)
    }

    pub fn get_node(&self, id: Uuid) -> Option<Node> {
        self.service.get_node(id)
    }

    pub fn list_nodes(&self) -> Vec<Node> {
        self.service.list_nodes()
    }

    pub fn list_edges(&self) -> Vec<Edge> {
        self.service.list_edges()
    }

    pub fn neighbors(&self, id: Uuid) -> Vec<(Edge, Option<Node>)> {
        self.service.neighbors(id)
    }

    pub fn remove_node(&self, id: Uuid) -> DkgResult<()> {
        self.service.remove_node(id)
    }

    pub fn remove_edge(&self, id: Uuid) -> DkgResult<()> {
        self.service.remove_edge(id)
    }
}