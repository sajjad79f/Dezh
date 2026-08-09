use uuid::Uuid;

use super::{AutonomyLevel, DecisionStatus};

#[derive(Debug, Clone)]
pub struct Decision {
    pub id: Uuid,
    pub subject: Uuid,
    pub description: String,
    pub autonomy: AutonomyLevel,
    pub status: DecisionStatus,
}

impl Decision {
    pub fn new(
        subject: Uuid,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
    ) -> Self {
        let status = match autonomy {
            AutonomyLevel::Critical => DecisionStatus::Alerted,
            AutonomyLevel::High => DecisionStatus::PendingApproval,
            AutonomyLevel::Low => DecisionStatus::AutoExecuted,
        };

        Self {
            id: Uuid::new_v4(),
            subject,
            description: description.into(),
            autonomy,
            status,
        }
    }
}