use std::collections::HashMap;
use uuid::Uuid;

use super::{AssetStatus, AssetType};

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub status: AssetStatus,
    pub metadata: HashMap<String, String>,
}

impl Asset {
    pub fn new(name: impl Into<String>, asset_type: AssetType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            asset_type,
            status: AssetStatus::Unknown,
            metadata: HashMap::new(),
        }
    }
}