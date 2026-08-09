use std::sync::Arc;
use uuid::Uuid;

use contracts::CoreService;
use dai::{Asset, AssetStatus};

use crate::error::DieResult;
use crate::models::{Finding, Severity};
use crate::registry::FindingRegistry;

pub struct DieService {
    registry: Arc<FindingRegistry>,
}

impl DieService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(FindingRegistry::new()),
        }
    }

    pub fn record_finding(
        &self,
        subject: Uuid,
        message: impl Into<String>,
        severity: Severity,
    ) -> DieResult<Uuid> {
        self.registry.record(subject, message, severity)
    }

    pub fn list_findings(&self) -> Vec<Finding> {
        self.registry.list()
    }

    pub fn get_finding(&self, id: Uuid) -> Option<Finding> {
        self.registry.get(id)
    }

    /// Rule: هر asset با status = Unknown یک Finding می‌گیرد.
    pub fn analyze_assets(&self, assets: &[Asset]) -> Vec<Finding> {
        assets
            .iter()
            .filter(|asset| asset.status == AssetStatus::Unknown)
            .filter_map(|asset| {
                let message = format!("Asset '{}' has unknown status", asset.name);

                self.registry
                    .record(asset.id, message.clone(), Severity::Warning)
                    .ok()
                    .map(|id| Finding {
                        id,
                        subject: asset.id,
                        message,
                        severity: Severity::Warning,
                    })
            })
            .collect()
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }

    pub fn clear(&self) {
        self.registry.clear();
    }
}

impl Clone for DieService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl CoreService for DieService {
    fn name(&self) -> &'static str {
        "DIE"
    }

    fn initialize(&self) {
        println!("[DIE] initialized");
    }

    fn shutdown(&self) {
        println!("[DIE] shutdown");
    }
}