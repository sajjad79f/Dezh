use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::error::{DaiError, DaiResult};
use crate::models::{Asset, AssetStatus, AssetType};

#[derive(Clone, Default)]
pub struct AssetRegistry {
    assets: Arc<RwLock<HashMap<Uuid, Asset>>>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &self,
        name: impl Into<String>,
        asset_type: AssetType,
    ) -> DaiResult<Uuid> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DaiError::InvalidName);
        }

        let asset = Asset::new(name, asset_type);
        let id = asset.id;

        self.assets
            .write()
            .expect("DAI registry lock poisoned")
            .insert(id, asset);

        Ok(id)
    }

    pub fn get(&self, id: Uuid) -> Option<Asset> {
        self.assets
            .read()
            .expect("DAI registry lock poisoned")
            .get(&id)
            .cloned()
    }

    pub fn list(&self) -> Vec<Asset> {
        self.assets
            .read()
            .expect("DAI registry lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn set_status(&self, id: Uuid, status: AssetStatus) -> DaiResult<()> {
        let mut guard = self.assets.write().expect("DAI registry lock poisoned");

        let asset = guard
            .get_mut(&id)
            .ok_or(DaiError::AssetNotFound(id))?;

        asset.status = status;
        Ok(())
    }

    pub fn remove(&self, id: Uuid) -> DaiResult<()> {
        self.assets
            .write()
            .expect("DAI registry lock poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or(DaiError::AssetNotFound(id))
    }

    pub fn count(&self) -> usize {
        self.assets
            .read()
            .expect("DAI registry lock poisoned")
            .len()
    }

    pub fn find_by_name(&self, name: &str) -> Option<Asset> {
        self.assets
            .read()
            .expect("DAI registry lock poisoned")
            .values()
            .find(|a| a.name == name)
            .cloned()
    }
}