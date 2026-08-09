use std::sync::Arc;
use uuid::Uuid;

use contracts::CoreService;

use crate::error::DaiResult;
use crate::models::{Asset, AssetStatus, AssetType};
use crate::registry::AssetRegistry;

pub struct DaiService {
    registry: Arc<AssetRegistry>,
}

impl DaiService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(AssetRegistry::new()),
        }
    }

    pub fn register_asset(
        &self,
        name: impl Into<String>,
        asset_type: AssetType,
    ) -> DaiResult<Uuid> {
        self.registry.register(name, asset_type)
    }

    pub fn get_asset(&self, id: Uuid) -> Option<Asset> {
        self.registry.get(id)
    }

    pub fn list_assets(&self) -> Vec<Asset> {
        self.registry.list()
    }

    pub fn set_status(&self, id: Uuid, status: AssetStatus) -> DaiResult<()> {
        self.registry.set_status(id, status)
    }

    pub fn remove_asset(&self, id: Uuid) -> DaiResult<()> {
        self.registry.remove(id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<Asset> {
        self.registry.find_by_name(name)
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }
}

impl Clone for DaiService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl CoreService for DaiService {
    fn name(&self) -> &'static str {
        "DAI"
    }

    fn initialize(&self) {
        println!("[DAI] initialized");
    }

    fn shutdown(&self) {
        println!("[DAI] shutdown");
    }
}