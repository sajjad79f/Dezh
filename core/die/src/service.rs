use std::sync::RwLock;

use contracts::CoreService;
use dai::{Asset, AssetStatus};
use uuid::Uuid;

use crate::models::{Finding, Severity};

/// Intelligence Engine -- turns raw asset/event data into findings.
///
/// `analyze_assets` is a first, deliberately simple rule: it does not
/// yet consume DEF events or DKG relationships (see ADR-0009 Related
/// items) -- it exists to prove the DAI -> DIE data path end to end.
/// More rules (event correlation, graph-aware analysis) get added here
/// without changing this service's public shape.
pub struct DieService {
    findings: RwLock<Vec<Finding>>,
}

impl DieService {
    pub fn new() -> Self {
        Self {
            findings: RwLock::new(Vec::new()),
        }
    }

    pub fn record_finding(
        &self,
        subject: Uuid,
        message: impl Into<String>,
        severity: Severity,
    ) -> Uuid {
        let id = Uuid::new_v4();

        let finding = Finding {
            id,
            subject,
            message: message.into(),
            severity,
        };

        self.findings
            .write()
            .expect("DIE findings lock poisoned")
            .push(finding);

        id
    }

    pub fn list_findings(&self) -> Vec<Finding> {
        self.findings
            .read()
            .expect("DIE findings lock poisoned")
            .clone()
    }

    /// Rule: any asset whose status is still `Unknown` is worth a
    /// finding, since it means DAI has never observed its real state.
    /// Returns the findings it recorded.
    pub fn analyze_assets(&self, assets: &[Asset]) -> Vec<Finding> {
        assets
            .iter()
            .filter(|asset| asset.status == AssetStatus::Unknown)
            .map(|asset| {
                self.record_and_return(
                    asset.id,
                    format!("Asset '{}' has unknown status", asset.name),
                    Severity::Warning,
                )
            })
            .collect()
    }

    fn record_and_return(
        &self,
        subject: Uuid,
        message: String,
        severity: Severity,
    ) -> Finding {
        let id = self.record_finding(subject, message.clone(), severity.clone());

        Finding { id, subject, message, severity }
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
