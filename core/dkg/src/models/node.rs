use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Uuid,
    pub label: String,
    pub properties: HashMap<String, String>,
}

impl Node {
    pub fn new(id: Uuid, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            properties: HashMap::new(),
        }
    }
}