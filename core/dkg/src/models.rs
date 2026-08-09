use std::collections::HashMap;

use uuid::Uuid;

/// A node in the graph. `label` is a free-form type name (e.g. "Server",
/// "Firewall") -- deliberately not an enum, since DKG must stay
/// schema-free (ADR-0009): new kinds of nodes must never require a code
/// change here.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: Uuid,
    pub label: String,
    pub properties: HashMap<String, String>,
}

/// A directed, typed edge between two nodes (e.g. "connected_to",
/// "protected_by"). Same schema-free reasoning as `Node::label`.
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub relation: String,
    pub properties: HashMap<String, String>,
}
