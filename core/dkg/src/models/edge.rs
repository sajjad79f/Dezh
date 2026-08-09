use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub relation: String,
    pub properties: HashMap<String, String>,
}

impl Edge {
    pub fn new(from: Uuid, to: Uuid, relation: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            relation: relation.into(),
            properties: HashMap::new(),
        }
    }
}