use std::sync::Arc;
use uuid::Uuid;

use dai::Asset;

use crate::error::DieResult;
use crate::models::{Finding, Severity};
use crate::service::DieService;

pub struct DieFacade {
    service: Arc<DieService>,
}

impl DieFacade {
    pub fn new() -> Self {
        Self {
            service: Arc::new(DieService::new()),
        }
    }

    pub fn from_service(service: Arc<DieService>) -> Self {
        Self { service }
    }

    pub fn record_finding(
        &self,
        subject: Uuid,
        message: impl Into<String>,
        severity: Severity,
    ) -> DieResult<Uuid> {
        self.service.record_finding(subject, message, severity)
    }

    pub fn list_findings(&self) -> Vec<Finding> {
        self.service.list_findings()
    }

    pub fn analyze_assets(&self, assets: &[Asset]) -> Vec<Finding> {
        self.service.analyze_assets(assets)
    }

    pub fn count(&self) -> usize {
        self.service.count()
    }
}