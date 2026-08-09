use std::sync::Arc;
use uuid::Uuid;

use crate::error::DaiResult;
use crate::models::{Asset, AssetStatus, AssetType};
use crate::service::DaiService;

/// Public entry point of DAI (similar to DefFacade).
pub struct DaiFacade {
    service: Arc<DaiService>,
}

impl DaiFacade {
    pub fn new() -> Self {
        Self {
            service: Arc::new(DaiService::new()),
        }
    }

    pub fn from_service(service: Arc<DaiService>) -> Self {
        Self { service }
    }

    pub fn register_asset(
        &self,
        name: impl Into<String>,
        asset_type: AssetType,
    ) -> DaiResult<Uuid> {
        self.service.register_asset(name, asset_type)
    }

    pub fn get_asset(&self, id: Uuid) -> Option<Asset> {
        self.service.get_asset(id)
    }

    pub fn list_assets(&self) -> Vec<Asset> {
        self.service.list_assets()
    }

    pub fn set_status(&self, id: Uuid, status: AssetStatus) -> DaiResult<()> {
        self.service.set_status(id, status)
    }

    pub fn remove_asset(&self, id: Uuid) -> DaiResult<()> {
        self.service.remove_asset(id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<Asset> {
        self.service.find_by_name(name)
    }

    pub fn count(&self) -> usize {
        self.service.count()
    }
}