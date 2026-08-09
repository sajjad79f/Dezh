use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    Endpoint,
    Server,
    VirtualMachine,
    Firewall,
    VpnGateway,
    User,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetStatus {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub status: AssetStatus,
    pub metadata: HashMap<String, String>,
}
