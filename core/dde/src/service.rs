use std::collections::HashMap;
use std::sync::RwLock;

use contracts::CoreService;
use uuid::Uuid;

use crate::models::{AutonomyLevel, Decision, DecisionStatus};

/// Decision Engine -- turns an intent ("do something about subject X")
/// into a `Decision`, whose initial status depends on its autonomy
/// level. Actually *carrying out* a Low-autonomy decision (calling into
/// a module's Adapter, per the Connector/Adapter direction discussed
/// earlier) is not wired yet -- `AutoExecuted` here only marks that DDE
/// would have acted, so the CLI/Web flow is testable end to end before
/// that execution path exists.
pub struct DdeService {
    decisions: RwLock<HashMap<Uuid, Decision>>,
}

impl DdeService {
    pub fn new() -> Self {
        Self {
            decisions: RwLock::new(HashMap::new()),
        }
    }

    pub fn submit_decision(
        &self,
        subject: Uuid,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
    ) -> Uuid {
        let id = Uuid::new_v4();

        let status = match autonomy {
            AutonomyLevel::Critical => DecisionStatus::Alerted,
            AutonomyLevel::High => DecisionStatus::PendingApproval,
            AutonomyLevel::Low => DecisionStatus::AutoExecuted,
        };

        let decision = Decision {
            id,
            subject,
            description: description.into(),
            autonomy,
            status,
        };

        self.decisions
            .write()
            .expect("DDE decision store lock poisoned")
            .insert(id, decision);

        id
    }

    pub fn approve(&self, id: Uuid) -> bool {
        self.transition(id, DecisionStatus::PendingApproval, DecisionStatus::Approved)
    }

    pub fn reject(&self, id: Uuid) -> bool {
        self.transition(id, DecisionStatus::PendingApproval, DecisionStatus::Rejected)
    }

    fn transition(
        &self,
        id: Uuid,
        expected: DecisionStatus,
        next: DecisionStatus,
    ) -> bool {
        let mut decisions = self
            .decisions
            .write()
            .expect("DDE decision store lock poisoned");

        match decisions.get_mut(&id) {
            Some(decision) if decision.status == expected => {
                decision.status = next;
                true
            }
            _ => false,
        }
    }

    pub fn list_decisions(&self) -> Vec<Decision> {
        self.decisions
            .read()
            .expect("DDE decision store lock poisoned")
            .values()
            .cloned()
            .collect()
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
