use std::sync::Arc;
use uuid::Uuid;

use crate::error::DdeResult;
use crate::models::{AutonomyLevel, Decision};
use crate::service::DdeService;

pub struct DdeFacade {
    service: Arc<DdeService>,
}

impl DdeFacade {
    pub fn new() -> Self {
        Self {
            service: Arc::new(DdeService::new()),
        }
    }

    pub fn from_service(service: Arc<DdeService>) -> Self {
        Self { service }
    }

    pub fn submit_decision(
        &self,
        subject: Uuid,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
    ) -> DdeResult<Uuid> {
        self.service.submit_decision(subject, description, autonomy)
    }

    pub fn approve(&self, id: Uuid) -> DdeResult<()> {
        self.service.approve(id)
    }

    pub fn reject(&self, id: Uuid) -> DdeResult<()> {
        self.service.reject(id)
    }

    pub fn list_decisions(&self) -> Vec<Decision> {
        self.service.list_decisions()
    }

    pub fn get_decision(&self, id: Uuid) -> Option<Decision> {
        self.service.get_decision(id)
    }
}