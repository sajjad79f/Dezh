use std::collections::HashMap;
use std::sync::RwLock;

use contracts::CoreService;
use uuid::Uuid;

use crate::models::{Asset, AssetStatus, AssetType};

/// Asset Intelligence — owns the platform's asset registry.
///
/// Uses a `RwLock` (interior mutability) instead of requiring `&mut self`
/// because, once registered, this service is shared through the
/// `ServiceContainer` as an immutable `&DaiService` (see ADR-0009 —
/// Core Services are resolved by shared reference, not owned/mutable).
pub struct DaiService {
    assets: RwLock<HashMap<Uuid, Asset>>,
}

impl DaiService {
    pub fn new() -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_asset(
        &self,
        name: impl Into<String>,
        asset_type: AssetType,
    ) -> Uuid {
        let id = Uuid::new_v4();

        let asset = Asset {
            id,
            name: name.into(),
            asset_type,
            status: AssetStatus::Unknown,
            metadata: HashMap::new(),
        };

        self.assets
            .write()
            .expect("DAI asset store lock poisoned")
            .insert(id, asset);

        id
    }

    pub fn get_asset(&self, id: Uuid) -> Option<Asset> {
        self.assets
            .read()
            .expect("DAI asset store lock poisoned")
            .get(&id)
            .cloned()
    }

    pub fn list_assets(&self) -> Vec<Asset> {
        self.assets
            .read()
            .expect("DAI asset store lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn set_status(&self, id: Uuid, status: AssetStatus) -> bool {
        if let Some(asset) = self
            .assets
            .write()
            .expect("DAI asset store lock poisoned")
            .get_mut(&id)
        {
            asset.status = status;
            true
        } else {
            false
        }
    }

    pub fn remove_asset(&self, id: Uuid) -> bool {
        self.assets
            .write()
            .expect("DAI asset store lock poisoned")
            .remove(&id)
            .is_some()
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
