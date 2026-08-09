use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::error::{DieError, DieResult};
use crate::models::{Finding, Severity};

#[derive(Clone, Default)]
pub struct FindingRegistry {
    findings: Arc<RwLock<Vec<Finding>>>,
}

impl FindingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(
        &self,
        subject: Uuid,
        message: impl Into<String>,
        severity: Severity,
    ) -> DieResult<Uuid> {
        let message = message.into();
        if message.trim().is_empty() {
            return Err(DieError::InvalidMessage);
        }

        let finding = Finding::new(subject, message, severity);
        let id = finding.id;

        self.findings
            .write()
            .expect("DIE findings lock poisoned")
            .push(finding);

        Ok(id)
    }

    pub fn list(&self) -> Vec<Finding> {
        self.findings
            .read()
            .expect("DIE findings lock poisoned")
            .clone()
    }

    pub fn get(&self, id: Uuid) -> Option<Finding> {
        self.findings
            .read()
            .expect("DIE findings lock poisoned")
            .iter()
            .find(|f| f.id == id)
            .cloned()
    }

    pub fn clear(&self) {
        self.findings
            .write()
            .expect("DIE findings lock poisoned")
            .clear();
    }

    pub fn count(&self) -> usize {
        self.findings
            .read()
            .expect("DIE findings lock poisoned")
            .len()
    }
}