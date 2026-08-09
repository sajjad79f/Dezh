use uuid::Uuid;

/// How much trust DDE has to act on its own for a given decision.
/// Matches the three-tier model agreed for the platform:
/// Critical -> alert only, a human must act.
/// High     -> DDE suggests an action, a human must approve it.
/// Low      -> DDE acts immediately, no approval needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyLevel {
    Critical,
    High,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionStatus {
    /// Critical: raised for a human to handle; DDE will never act on it.
    Alerted,
    /// High: waiting for a human to approve or reject.
    PendingApproval,
    Approved,
    Rejected,
    /// Low: DDE already carried this out.
    AutoExecuted,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub id: Uuid,
    pub subject: Uuid,
    pub description: String,
    pub autonomy: AutonomyLevel,
    pub status: DecisionStatus,
}
