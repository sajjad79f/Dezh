use std::sync::Arc;
use uuid::Uuid;

use contracts::CoreService;

use crate::error::DdeResult;
use crate::models::{AutonomyLevel, Decision};
use crate::registry::DecisionRegistry;

pub struct DdeService {
    registry: Arc<DecisionRegistry>,
}

impl DdeService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(DecisionRegistry::new()),
        }
    }

    pub fn submit_decision(
        &self,
        subject: Uuid,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
    ) -> DdeResult<Uuid> {
        self.registry.submit(subject, description, autonomy)
    }

    pub fn approve(&self, id: Uuid) -> DdeResult<()> {
        self.registry.approve(id)
    }

    pub fn reject(&self, id: Uuid) -> DdeResult<()> {
        self.registry.reject(id)
    }

    pub fn get_decision(&self, id: Uuid) -> Option<Decision> {
        self.registry.get(id)
    }

    pub fn list_decisions(&self) -> Vec<Decision> {
        self.registry.list()
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }
}

impl Clone for DdeService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl CoreService for DdeService {
    fn name(&self) -> &'static str {
        "DDE"
    }

    fn initialize(&self) {
        println!("[DDE] initialized");
    }

    fn shutdown(&self) {
        println!("[DDE] shutdown");
    }
}