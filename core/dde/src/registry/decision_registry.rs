use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::error::{DdeError, DdeResult};
use crate::models::{AutonomyLevel, Decision, DecisionStatus};

#[derive(Clone, Default)]
pub struct DecisionRegistry {
    decisions: Arc<RwLock<HashMap<Uuid, Decision>>>,
}

impl DecisionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(
        &self,
        subject: Uuid,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
    ) -> DdeResult<Uuid> {
        let description = description.into();
        if description.trim().is_empty() {
            return Err(DdeError::InvalidDescription);
        }

        let decision = Decision::new(subject, description, autonomy);
        let id = decision.id;

        self.decisions
            .write()
            .expect("DDE decision store lock poisoned")
            .insert(id, decision);

        Ok(id)
    }

    pub fn approve(&self, id: Uuid) -> DdeResult<()> {
        self.transition(id, DecisionStatus::PendingApproval, DecisionStatus::Approved)
    }

    pub fn reject(&self, id: Uuid) -> DdeResult<()> {
        self.transition(id, DecisionStatus::PendingApproval, DecisionStatus::Rejected)
    }

    fn transition(
        &self,
        id: Uuid,
        expected: DecisionStatus,
        next: DecisionStatus,
    ) -> DdeResult<()> {
        let mut decisions = self
            .decisions
            .write()
            .expect("DDE decision store lock poisoned");

        match decisions.get_mut(&id) {
            Some(decision) if decision.status == expected => {
                decision.status = next;
                Ok(())
            }
            Some(_) => Err(DdeError::InvalidTransition(id)),
            None => Err(DdeError::DecisionNotFound(id)),
        }
    }

    pub fn get(&self, id: Uuid) -> Option<Decision> {
        self.decisions
            .read()
            .expect("DDE decision store lock poisoned")
            .get(&id)
            .cloned()
    }

    pub fn list(&self) -> Vec<Decision> {
        self.decisions
            .read()
            .expect("DDE decision store lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn count(&self) -> usize {
        self.decisions
            .read()
            .expect("DDE decision store lock poisoned")
            .len()
    }
}