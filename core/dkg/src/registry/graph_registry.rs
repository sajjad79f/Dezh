use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::error::{DkgError, DkgResult};
use crate::models::{Edge, Node};

#[derive(Clone, Default)]
pub struct GraphRegistry {
    nodes: Arc<RwLock<HashMap<Uuid, Node>>>,
    edges: Arc<RwLock<HashMap<Uuid, Edge>>>,
}

impl GraphRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&self, id: Uuid, label: impl Into<String>) -> DkgResult<()> {
        let label = label.into();
        if label.trim().is_empty() {
            return Err(DkgError::InvalidLabel);
        }

        let node = Node::new(id, label);

        self.nodes
            .write()
            .expect("DKG node store lock poisoned")
            .insert(id, node);

        Ok(())
    }

    pub fn add_edge(
        &self,
        from: Uuid,
        to: Uuid,
        relation: impl Into<String>,
    ) -> DkgResult<Uuid> {
        let edge = Edge::new(from, to, relation);
        let id = edge.id;

        self.edges
            .write()
            .expect("DKG edge store lock poisoned")
            .insert(id, edge);

        Ok(id)
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

    pub fn neighbors(&self, id: Uuid) -> Vec<(Edge, Option<Node>)> {
        let edges = self.edges.read().expect("DKG edge store lock poisoned");
        let nodes = self.nodes.read().expect("DKG node store lock poisoned");

        edges
            .values()
            .filter(|edge| edge.from == id)
            .map(|edge| (edge.clone(), nodes.get(&edge.to).cloned()))
            .collect()
    }

    pub fn remove_node(&self, id: Uuid) -> DkgResult<()> {
        let mut nodes = self.nodes.write().expect("DKG node store lock poisoned");
        let mut edges = self.edges.write().expect("DKG edge store lock poisoned");

        if nodes.remove(&id).is_none() {
            return Err(DkgError::NodeNotFound(id));
        }

        edges.retain(|_, edge| edge.from != id && edge.to != id);
        Ok(())
    }

    pub fn remove_edge(&self, id: Uuid) -> DkgResult<()> {
        self.edges
            .write()
            .expect("DKG edge store lock poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or(DkgError::EdgeNotFound(id))
    }

    pub fn node_count(&self) -> usize {
        self.nodes
            .read()
            .expect("DKG node store lock poisoned")
            .len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges
            .read()
            .expect("DKG edge store lock poisoned")
            .len()
    }
}